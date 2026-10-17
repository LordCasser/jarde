#!/usr/bin/env python3
"""Collect a focused Local fixture Jarde/JADX source and runtime comparison.

The four positional arguments are repository root, a fresh output directory, the final CLI,
and its receipt. Uses the sibling OpenSpec compare.py and its pinned JDK/guard helpers.
"""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import shutil
import sys

ROOT_CLI = Path('/private/tmp/jarde-exception-only-final-focused-root-v3-20261018/candidate-cli')
CLI_SHA256 = '7287e856c5ce256c127fc6b33bcc3c663b058e913bf50de5d82ed9c9373b95c8'
CLI_RECEIPT_SHA256 = CLI_SHA256
LOCAL_CLASS_REL = Path('tests/fixtures/recover-exception-only-finally-completion/v8/ExceptionOnlyLocal.class')
LOCAL_CLASS_SHA256 = 'c5c1680de3820dd022b29a5a4aad8b13f0c877a65cabaecf4702606a81b8bc73'
JADX = Path('/opt/homebrew/Cellar/jadx/1.5.6/bin/jadx')
JADX_SHA256 = '64a6ee6bcf7490ea682508db2a73d6cda8b671a5211af5ee3ff098441af038a7'
JADX_JAR = Path('/opt/homebrew/Cellar/jadx/1.5.6/libexec/lib/jadx-1.5.6-all.jar')
JADX_JAR_SHA256 = '966d314282d95ff7c72f597bb9c744999856666a3b856575e3c035a9c12a0b65'
MANIFEST_REL = Path('openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json')
MANIFEST_SHA256 = 'ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec'
GUARD_REL = Path('openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py')
GUARD_SHA256 = '51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
SCANNER_REL = Path('openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py')
SCANNER_SHA256 = 'd954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7'
METHODS = {('<init>', '()V'), ('mark', '(I)V'), ('run', '(Ljava/lang/RuntimeException;)I')}

DRIVER_PATH = Path(__file__).with_name('ExceptionOnlyLocalDriver.java')
EXPECTED_LINES = (
    'null:trace=78:type=java.lang.NullPointerException',
    'state:trace=78:identity=true:type=java.lang.IllegalStateException:message=local-state',
    'argument:trace=78:identity=true:type=java.lang.IllegalArgumentException:message=local-argument',
)
ENV_STRIP = ('JAVA_TOOL_OPTIONS', '_JAVA_OPTIONS', 'JDK_JAVA_OPTIONS', 'CLASSPATH', 'CARGO_TARGET_DIR')


def load_module(path: Path, name: str):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f'cannot load helper: {path}')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def ensure(condition: bool, message: str) -> None:
    if not condition:
        raise RuntimeError(message)


def local_report_origins(doc: dict, facts: dict) -> dict:
    """Require complete class identity and every original Code BCI in source evidence."""
    rows = doc.get('methods')
    ensure(isinstance(rows, list), 'class-source JSON has no method list')
    by_key = {}
    for row in rows:
        item = row.get('item', {})
        identity = item.get('identity', {})
        try:
            key = (bytes(identity['name']).decode('utf-8'), bytes(identity['descriptor']).decode('utf-8'))
        except (KeyError, TypeError, UnicodeDecodeError) as exc:
            raise RuntimeError(f'method identity is missing UTF-8 bytes: {exc}') from exc
        if key in by_key:
            raise RuntimeError(f'duplicate method identity in class-source report: {key}')
        by_key[key] = row
        owner = identity.get('owner', {}).get('class_bytes', {})
        if owner.get('digest') != facts['blake3'] or owner.get('length') != facts['bytes']:
            raise RuntimeError(f'method physical class identity differs from frozen input: {key}')
        name_raw = bytes(item.get('name', {}).get('raw', []))
        desc_raw = bytes(item.get('descriptor', {}).get('raw', []))
        if name_raw != key[0].encode('utf-8') or desc_raw != key[1].encode('utf-8'):
            raise RuntimeError(f'method name/descriptor UTF-8 spelling differs from identity: {key}')
    expected = {(m['name'], m['descriptor']) for m in facts['methods']}
    ensure(set(by_key) == expected, f'method identity set differs: expected={sorted(expected)}, got={sorted(by_key)}')
    class_identity = doc.get('declaration', {}).get('item', {}).get('declaration', {}).get('this_class', {})
    if bytes(class_identity.get('raw', [])).decode('utf-8') != facts['class_name']:
        raise RuntimeError('class declaration UTF-8 identity differs from frozen classfile')
    if doc.get('execution', {}).get('status') != 'complete':
        raise RuntimeError('class-source analysis did not complete')
    origin_sets = {}
    for method in facts['methods']:
        key = (method['name'], method['descriptor'])
        row = by_key[key]
        ensure(row.get('outcome', {}).get('kind') == 'recovered', f'{key} did not recover')
        report = row.get('outcome', {}).get('report', {})
        ensure(not report.get('fallbacks'), f'{key} contains source fallback')
        code = method.get('code')
        ensure(code is not None, f'{key} has no Code attribute')
        ensure(report.get('quality')=='structured' and report.get('representation')=='java', f'{key}: incomplete Java body')
        source_map = report.get('source_map', {})
        text=report['text'].encode('utf-8')
        bcis = set()
        for segment in source_map.get('segments', []):
            lo,hi=segment['start'],segment['end'];ensure(0<=lo<hi<=len(text),f'{key}: source-map bounds');text[:lo].decode('utf-8');text[:hi].decode('utf-8')
            origin = segment.get('origin', {})
            for point in [origin['primary'],*origin['derived']]:
                if point.get('bci') is not None:ensure(point['method']==row['item']['identity'],f'{key}: foreign physical origin')
            primary = origin.get('primary')
            if isinstance(primary, dict) and isinstance(primary.get('bci'), int):
                bcis.add(primary['bci'])
            for derived in origin.get('derived', []):
                if isinstance(derived.get('bci'), int):
                    bcis.add(derived['bci'])
        expected_bcis = set(code['bcis'])
        ensure(bcis == expected_bcis,
               f'{key} BCI origin mismatch: missing={sorted(expected_bcis-bcis)}, extra={sorted(bcis-expected_bcis)}')
        origin_sets[f'{key[0]}{key[1]}'] = sorted(bcis)
    return origin_sets


def valid_stdout(data: bytes, label: str) -> None:
    lines = data.decode('utf-8', errors='strict').splitlines()
    ensure(tuple(lines) == EXPECTED_LINES, f'{label} output differs from deterministic trace/identity oracle: {lines!r}')


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('root', type=Path)
    ap.add_argument('out', type=Path)
    ap.add_argument('cli', type=Path)
    ap.add_argument('receipt', type=Path)
    args = ap.parse_args()
    root = args.root.resolve(strict=True)
    out = args.out.resolve()
    cli = args.cli.resolve(strict=True)
    receipt_path = args.receipt.resolve(strict=True)
    ensure(cli == ROOT_CLI and hashlib.sha256(cli.read_bytes()).hexdigest() == CLI_SHA256,
           'CLI path or digest differs from final-focused receipt pin')
    receipt = json.loads(receipt_path.read_text(encoding='utf-8'))
    ensure(receipt.get('status') == 'commands_passed_pending_independent_acceptance',
           'CLI receipt is not the final focused pending-acceptance receipt')
    compare = load_module(root / 'openspec/changes/recover-exception-only-finally-completion/tools/compare.py', 'local_compare_helpers')
    ensure(compare.cli_receipt_digest(receipt, cli) == CLI_RECEIPT_SHA256,
           'CLI receipt does not bind the supplied exact CLI digest')
    ensure(not out.exists(), f'refusing to overwrite output: {out}')
    ensure(root not in out.parents and out != root, 'output must be outside repository root')
    ensure(not (root / 'target').exists(), 'refusing to run while repository target exists')
    local_class = root / LOCAL_CLASS_REL
    ensure(hashlib.sha256(local_class.read_bytes()).hexdigest() == LOCAL_CLASS_SHA256,
           'frozen 367-byte ExceptionOnlyLocal input digest changed')
    jadx = JADX; jadx_jar = JADX_JAR
    ensure(jadx.is_file() and jadx_jar.is_file(), 'pinned JADX executable or JAR is missing')
    ensure(hashlib.sha256(jadx.read_bytes()).hexdigest() == JADX_SHA256, 'JADX executable pin mismatch')
    ensure(hashlib.sha256(jadx_jar.read_bytes()).hexdigest() == JADX_JAR_SHA256, 'JADX JAR pin mismatch')
    manifest = (root / MANIFEST_REL).resolve(strict=True)
    jdks, jdk_record = compare.configured_jdks(manifest, MANIFEST_SHA256)
    guard_path = (root / GUARD_REL).resolve(strict=True)
    scanner_path = (root / SCANNER_REL).resolve(strict=True)
    ensure(compare.sha(guard_path) == GUARD_SHA256 and compare.sha(scanner_path) == SCANNER_SHA256,
           'runner guard/scanner pin mismatch')
    guard = load_module(guard_path, 'local_runtime_guard')
    scanner = load_module(scanner_path, 'local_runtime_scanner')
    out.mkdir(parents=True)
    raw = out / 'raw'; raw.mkdir()
    guard.ROOT = root
    guard.OUT = raw
    guard.target_bytes = lambda: scanner.target_bytes(root)
    guard.command_stream = lambda path: compare.pin(path)
    guard.expected_test_summaries = lambda index, stdout_bytes: None
    driver_text = DRIVER_PATH.read_text(encoding='utf-8')
    source_before = compare.source_closure(root)
    original_facts = compare.expected_methods(local_class, METHODS)
    fingerprint=json.loads((root/'tests/fixtures/corpus-fingerprint.json').read_text());fp=next(v for v in fingerprint['files'] if v['path']==LOCAL_CLASS_REL.as_posix());ensure(fp['bytes']==original_facts['bytes'],'corpus length');original_facts['blake3']=fp['blake3']
    rows = []
    result = {
        'schema': 'exception-only-local-runtime-v1', 'status': 'running',
        'root': str(root), 'output': str(out), 'input_class': compare.pin(local_class),
        'input_class_facts': original_facts, 'cli': compare.pin(cli),
        'cli_receipt': compare.pin(receipt_path), 'jadx': compare.pin(jadx), 'jadx_jar': compare.pin(jadx_jar),
        'jdk_manifest': compare.pin(manifest), 'jdk_manifest_record': jdk_record,
        'guard': compare.pin(guard_path), 'scanner': compare.pin(scanner_path),
        'driver_source': compare.pin(DRIVER_PATH), 'source_pins_before': source_before,
        'commands': rows, 'oracle_runs': [], 'jarde_outputs': [], 'jadx_outputs': [],
    }
    compare.write_json(out / 'execution.json', result)
    env_base = compare.clean_env(dict(os.environ))

    def cmd(label: str, argv: list[str], leg: str | None = None, phase: str = '') -> dict:
        env = compare.clean_env(env_base, jdks[leg] if leg else None)
        row = guard.run_command(len(rows) + 1, argv, env)
        row.update({'label': label, 'phase': phase})
        rows.append(row)
        compare.write_json(out / 'execution.json', result)
        if row.get('guard_stop') is not None:
            raise RuntimeError(f'disk/target guard stopped command {label}: {row["guard_stop"]}')
        return row

    def stdout(row: dict) -> bytes:
        return compare.command_artifact(row, 'stdout')

    def stderr(row: dict) -> bytes:
        return compare.command_artifact(row, 'stderr')

    def fresh_dir(path: Path) -> None:
        ensure(not path.exists(), f'refusing existing output path: {path}')
        path.mkdir(parents=True)

    def compile_java(source: Path, leg: str, output: Path, cp: list[Path], label: str) -> dict:
        output.parent.mkdir(parents=True, exist_ok=True)
        fresh_dir(output)
        empty = output.parent / (output.name + '-empty-sourcepath')
        fresh_dir(empty)
        cp_value = os.pathsep.join(str(p) for p in cp) if cp else str(empty)
        flags = (['-source', '8', '-target', '8', '-g:none', '-Xlint:-options', '-proc:none']
                 if leg == 'javac8' else ['--release', '8', '-g:none', '-proc:none'])
        return cmd(label, [str(jdks[leg]['tools']['javac']), *flags, '-classpath', cp_value,
                           '-sourcepath', str(empty), '-d', str(output), str(source)], leg, 'javac')

    def run_java(label: str, class_dir: Path, driver_dir: Path, runtime: str) -> dict:
        classpath = os.pathsep.join((str(class_dir), str(driver_dir)))
        return cmd(label, [str(jdks[runtime]['tools']['java']), '-Xverify:all', '-cp', classpath,
                           'ExceptionOnlyLocalDriver'], runtime, 'runtime')

    try:
        # Oracle is the exact frozen input class, with only the test driver on the other classpath root.
        oracle: dict[str, bytes] = {}
        original_root = out / 'original' / 'target'; original_root.mkdir(parents=True)
        shutil.copyfile(local_class, original_root / 'ExceptionOnlyLocal.class')
        for leg in ('javac8', 'javac23'):
            driver_path = out / 'drivers' / f'original-{leg}'
            driver_src = out / 'drivers-src' / 'ExceptionOnlyLocalDriver.java'
            driver_src.parent.mkdir(parents=True, exist_ok=True)
            if not driver_src.exists(): driver_src.write_text(driver_text, encoding='utf-8')
            driver_compile = compile_java(driver_src, leg, driver_path, [original_root], f'original-driver-{leg}')
            driver_class = driver_path / 'ExceptionOnlyLocalDriver.class'
            ensure(driver_compile['exit_code'] == 0 and driver_class.is_file(), f'original driver compile failed: {leg}')
            for runtime in ('javac8', 'javac23'):
                run = run_java(f'original-{runtime}', original_root, driver_path, runtime)
                ensure(run['exit_code'] == 0, f'frozen original runtime failed: {runtime}')
                data = stdout(run); valid_stdout(data, f'original/{runtime}')
                if runtime in oracle:ensure(oracle[runtime]==data,'original compiler driver oracle mismatch')
                oracle[runtime] = data
                result['oracle_runs'].append({'runtime': runtime, 'command': run['index'],
                    'stdout_sha256': run['streams']['stdout']['sha256'],
                    'stderr_sha256': run['streams']['stderr']['sha256'], 'oracle_valid': True})
                compare.write_json(out / 'execution.json', result)

        # One class-source report per evidence mode; save the CLI text exactly, without edits.
        docs = {}
        for evidence in ('default', 'all'):
            row = cmd(f'jarde-{evidence}', [str(cli), 'class-source', '--input', str(local_class),
                '--class', 'ExceptionOnlyLocal', '--policy', 'single-class', '--release', '8', '--format', 'json',
                *(['--evidence', 'all'] if evidence == 'all' else [])], phase='jarde-class-source')
            ensure(row['exit_code'] == 0, f'Jarde class-source failed: {evidence}')
            doc = json.loads(stdout(row).decode('utf-8'))
            text = doc.get('text')
            ensure(isinstance(text, str), f'{evidence}: class-source JSON has no source text')
            origins = local_report_origins(doc, original_facts)
            doc_path = out / 'jarde-json' / f'{evidence}.json'; doc_path.parent.mkdir(parents=True, exist_ok=True)
            doc_path.write_bytes(stdout(row))
            source_path = out / 'jarde-source' / evidence / 'ExceptionOnlyLocal.java'
            source_path.parent.mkdir(parents=True, exist_ok=True)
            source_path.write_bytes(text.encode('utf-8'))
            if evidence == 'all':
                ensure(source_path.read_bytes()==Path(docs['default']['source']['path']).read_bytes(),'default/all source text differs')
                default_doc=json.loads(Path(docs['default']['json']['path']).read_text())
                for old,new in zip(default_doc['methods'],doc['methods']):
                    ensure(old['item']==new['item'],'default/all member identity differs')
                    for field in ['text','source_map','quality','representation','content','fallbacks']:
                        ensure(old['outcome']['report'][field]==new['outcome']['report'][field],f'default/all {field} differs')
            item = {'evidence': evidence, 'command': row['index'], 'json': compare.pin(doc_path),
                    'source': compare.pin(source_path), 'source_text_sha256': compare.sha(source_path),
                    'method_bci_origins': origins, 'methods': sorted([list(x) for x in METHODS])}
            docs[evidence] = item
            result['jarde_outputs'].append(item)
            compare.write_json(out / 'execution.json', result)
        ensure(docs['default']['method_bci_origins'] == docs['all']['method_bci_origins'],
               'default/all method source-map origins differ')

        generated_runs = []
        for evidence in ('default','all'):
            generated=Path(docs[evidence]['source']['path'])
            for leg in ('javac8', 'javac23'):
                target_root = out / 'jarde-compiled' / evidence / leg / 'classes'
                comp = compile_java(generated, leg, target_root, [], f'jarde-source-compile-{evidence}-{leg}')
                class_file = target_root / 'ExceptionOnlyLocal.class'
                row = {'compiler': leg, 'compile_command': comp['index'], 'exit_code': comp['exit_code'],
                       'stderr_sha256': comp['streams']['stderr']['sha256'], 'class': None, 'driver': None, 'runs': []}
                if comp['exit_code'] == 0 and class_file.is_file():
                    row['class'] = compare.pin(class_file)
                    compiled_facts = compare.expected_methods(class_file, METHODS)
                    row['class_facts'] = compiled_facts
                    driver_dir = out / 'jarde-driver-classes' / evidence / leg
                    driver_compile = compile_java(driver_src, leg, driver_dir, [target_root], f'jarde-driver-compile-{evidence}-{leg}')
                    driver_class = driver_dir / 'ExceptionOnlyLocalDriver.class'
                    row['driver'] = {'command': driver_compile['index'], 'exit_code': driver_compile['exit_code'],
                                     'class': compare.pin(driver_class) if driver_class.is_file() else None}
                    if driver_compile['exit_code'] == 0 and driver_class.is_file():
                        for runtime in ('javac8', 'javac23'):
                            run = run_java(f'jarde-{evidence}-{leg}-on-{runtime}', target_root, driver_dir, runtime)
                            data = stdout(run) if run['exit_code'] == 0 else b''
                            valid = False
                            if run['exit_code'] == 0:
                                valid_stdout(data, f'Jarde {leg}/{runtime}')
                                valid = data == oracle[runtime]
                            row['runs'].append({'runtime': runtime, 'command': run['index'],
                                'exit_code': run['exit_code'], 'stdout_sha256': run['streams']['stdout']['sha256'],
                                'stderr_sha256': run['streams']['stderr']['sha256'],
                                'matches_same_runtime_original_oracle': valid})
                            ensure(run['exit_code'] == 0 and valid,
                                   f'Jarde rebuilt-only runtime differs from original: {leg}/{runtime}')
                ensure(comp['exit_code'] == 0 and class_file.is_file(), f'generated full source did not compile: {leg}')
                ensure(row['driver'] is not None and row['driver']['exit_code'] == 0,
                       f'generated source driver did not compile: {leg}')
                generated_runs.append(row)
                docs[evidence].setdefault('compiler_runs', []).append(row)
                compare.write_json(out / 'execution.json', result)

        # JADX is an observation gate: preserve its source, compile failures and any runnable outputs.
        version = cmd('jadx-version', [str(jadx), '--version'], 'javac23', 'jadx-version')
        version_text = (stdout(version) + stderr(version)).decode('utf-8', errors='replace')
        ensure('1.5.6' in version_text, f'JADX version output does not identify 1.5.6: {version_text!r}')
        jadx_dir = out / 'jadx'; jadx_dir.parent.mkdir(parents=True, exist_ok=True)
        ensure(not jadx_dir.exists(), f'refusing existing JADX output {jadx_dir}')
        jrow = cmd('jadx-decompile-original', [str(jadx), '--config', 'none', '--threads-count', '1',
            '--no-res', '--rename-flags', 'none', '-d', str(jadx_dir), str(local_class)], 'javac23', 'jadx-source')
        jadx_record = {'decompile_command': jrow['index'], 'exit_code': jrow['exit_code'],
                       'source': None, 'compiles': []}
        hits = list(jadx_dir.rglob('ExceptionOnlyLocal.java')) if jrow['exit_code'] == 0 else []
        ensure(len(hits) == 1, f'JADX did not emit exactly one ExceptionOnlyLocal.java: found {len(hits)}')
        jadx_source = hits[0]
        jadx_record['source'] = compare.pin(jadx_source)
        raw_source = jadx_source.read_bytes()
        package_match = re.search(rb'^\s*package\s+([\w.]+)\s*;', raw_source, re.M)
        package = package_match.group(1).decode('ascii') if package_match else ''
        jadx_record['package'] = package
        for leg in ('javac8', 'javac23'):
            target_root = out / 'jadx-compiled' / leg / 'classes'
            comp = compile_java(jadx_source, leg, target_root, [], f'jadx-source-compile-{leg}')
            target_class = target_root / ((package.replace('.', '/') + '/') if package else '') / 'ExceptionOnlyLocal.class'
            item = {'compiler': leg, 'compile_command': comp['index'], 'exit_code': comp['exit_code'],
                    'stderr_sha256': comp['streams']['stderr']['sha256'],
                    'class': compare.pin(target_class) if target_class.is_file() else None,
                    'driver': None, 'runs': []}
            if comp['exit_code'] == 0 and target_class.is_file():
                driver_path = out / 'jadx-adapted-driver' / leg / 'ExceptionOnlyLocalDriver.java'
                driver_path.parent.mkdir(parents=True, exist_ok=True)
                driver_source = driver_text if not package else f'package {package};\n\n' + driver_text
                driver_path.write_text(driver_source, encoding='utf-8')
                driver_dir = out / 'jadx-driver-classes' / leg
                driver_compile = compile_java(driver_path, leg, driver_dir, [target_root], f'jadx-driver-compile-{leg}')
                driver_class = driver_dir / ((package.replace('.', '/') + '/') if package else '') / 'ExceptionOnlyLocalDriver.class'
                item['driver'] = {'command': driver_compile['index'], 'exit_code': driver_compile['exit_code'],
                                  'source': compare.pin(driver_path),
                                  'class': compare.pin(driver_class) if driver_class.is_file() else None,
                                  'package_prefix_only': package != ''}
                if driver_compile['exit_code'] == 0 and driver_class.is_file():
                    main = f'{package}.ExceptionOnlyLocalDriver' if package else 'ExceptionOnlyLocalDriver'
                    for runtime in ('javac8', 'javac23'):
                        cp = os.pathsep.join((str(target_root), str(driver_dir)))
                        run = cmd(f'jadx-{leg}-on-{runtime}', [str(jdks[runtime]['tools']['java']),
                            '-Xverify:all', '-cp', cp, main], runtime, 'jadx-rebuilt-runtime')
                        data = stdout(run) if run['exit_code'] == 0 else b''
                        valid = False; matches = False; validation_error = None
                        if run['exit_code'] == 0:
                            try:
                                valid_stdout(data, f'JADX {leg}/{runtime}')
                                valid = True; matches = data == oracle[runtime]
                            except Exception as exc:
                                validation_error = f'{type(exc).__name__}: {exc}'
                        item['runs'].append({'runtime': runtime, 'command': run['index'],
                            'exit_code': run['exit_code'], 'stdout_sha256': run['streams']['stdout']['sha256'],
                            'stderr_sha256': run['streams']['stderr']['sha256'],
                            'oracle_shape_valid': valid, 'matches_same_runtime_original_oracle': matches,
                            'validation_error': validation_error})
            jadx_record['compiles'].append(item)
            result['jadx_outputs'] = [jadx_record]
            compare.write_json(out / 'execution.json', result)

        source_after = compare.source_closure(root)
        ensure(source_before == source_after, 'repository source closure changed during collection')
        result['source_unchanged'] = True
        result['target_absent']=not(root/'target').exists();ensure(result['target_absent'],'unexpected target')
        result['status'] = 'commands complete; Jarde full-source runtime matched each JVM oracle; JADX outcome recorded'
        compare.write_json(out / 'execution.json', result)
        print(json.dumps({'status': result['status'], 'commands': len(rows),
                          'original_oracles': len(result['oracle_runs']),
                          'jarde_profiles': len(result['jarde_outputs']),
                          'jarde_compiler_runtime_runs': sum(len(c.get('runs', [])) for c in generated_runs),
                          'jadx_compiler_legs': len(jadx_record['compiles']),
                          'output': str(out)}, indent=2))
        return 0
    except Exception as exc:
        result['status'] = 'collector failed; partial raw evidence preserved'
        result['failure'] = f'{type(exc).__name__}: {exc}'
        compare.write_json(out / 'execution.json', result)
        raise


if __name__ == '__main__':
    try:
        raise SystemExit(main())
    except (OSError, RuntimeError, ValueError, KeyError, json.JSONDecodeError) as exc:
        print(f'Local runtime comparison failed: {exc}', file=sys.stderr)
        raise SystemExit(1)
