#!/usr/bin/env python3
"""Independent, read-only review of one captured TestNestedLoops5 baseline receipt."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import sys
import zipfile

ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
PREP = Path('/private/tmp/jarde-testnestedloops5-baseline-prep-root-v3-20261018')
COMPARE = ROOT / 'openspec/changes/recover-exception-only-finally-completion/tools/compare.py'
SOURCE = Path('/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/tests/integration/loops/TestNestedLoops5.java')
SOURCE_SHA = '0be29d145607e1be1d1426305739e6879d76defc8472dff7f6e82fcec0d83b6d'
CLI = Path('/private/tmp/jarde-prefixed-retry-final-focused-root-v5-20261018/candidate-cli')
CLI_SHA = '25adeaf312a7644c7d4c73191ef1264af0378cf78139d35a890377e3c86371ce'
JADX = Path('/opt/homebrew/Cellar/jadx/1.5.6/bin/jadx')
JADX_SHA = '64a6ee6bcf7490ea682508db2a73d6cda8b671a5211af5ee3ff098441af038a7'
JADX_JAR = Path('/opt/homebrew/Cellar/jadx/1.5.6/libexec/lib/jadx-1.5.6-all.jar')
JADX_JAR_SHA = '966d314282d95ff7c72f597bb9c744999856666a3b856575e3c035a9c12a0b65'
DEPENDENCY_REFERENCE = Path('/private/tmp/jarde-nestedloops3-baseline-root-v1-20261018/execution.json')
DEPENDENCY_REFERENCE_SHA = '5a7d6572e600d19dc45d78030d0b12867bc742c4b014381fb5c7b059db171d35'
JDK_MANIFEST = ROOT / 'openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json'
JDK_MANIFEST_SHA = 'ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec'
GUARD = ROOT / 'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
GUARD_SHA = '51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
SCANNER = ROOT / 'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py'
SCANNER_SHA = 'd954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7'
RUNTIME_BOUND = Path('/private/tmp/jarde-nestedloops3-runtime-bound-root-v1-20261018.py')
INTERNAL_CLASS = 'jadx/tests/integration/loops/TestNestedLoops5$TestCls'
OUTER_CLASS = 'jadx/tests/integration/loops/TestNestedLoops5'
METHODS = [('<init>', '()V', 1), ('testFor', '()I', 1), ('check', '()V', 1)]
PROFILES = ('debug', 'no-debug')
RUNTIMES = ('javac8', 'javac23')
FORMS = ('child', 'bundle-child')
EVIDENCE = ('default', 'all')


def need(ok: bool, message: str) -> None:
    if not ok:
        raise ValueError(message)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def file_pin(path: Path) -> dict:
    path = path.resolve(strict=True)
    data = path.read_bytes()
    return {'path': str(path), 'bytes': len(data), 'sha256': sha(data)}


def verify_pin(record: dict, label: str) -> bytes:
    path = Path(record.get('path', ''))
    need(path.is_absolute() and path.is_file(), f'{label}: missing absolute artifact: {path}')
    data = path.read_bytes()
    need(len(data) == record.get('bytes') and sha(data) == record.get('sha256'),
         f'{label}: bytes/SHA-256 differ from receipt')
    return data


def byte_boundaries(data: bytes) -> set[int]:
    return {0, len(data), *(i for i, byte in enumerate(data) if byte & 0xC0 != 0x80)}


def method_facts(profile: dict) -> list[dict]:
    return profile['target']['methods']


def bci_set(fact: dict) -> set[int]:
    code = fact.get('code')
    need(isinstance(code, dict), f"{fact.get('name')}{fact.get('descriptor')}: no Code attribute")
    raw = bytes.fromhex(code['code_hex'])
    need(len(raw) == code.get('code_length'), 'Code hex length differs from declared code_length')
    bcis = code.get('bcis')
    need(isinstance(bcis, list) and bcis and bcis[0] == 0 and bcis == sorted(set(bcis)), 'Code BCI list is malformed')
    need(all(isinstance(bci, int) and 0 <= bci < len(raw) for bci in bcis), 'Code BCI outside Code length')
    need(bcis == decode_bcis(raw), f"{fact.get('name')}{fact.get('descriptor')}: physical BCI census disagrees with Code bytes")
    return set(bcis)


def decode_bcis(code: bytes) -> list[int]:
    fixed = {0x10: 2, 0x11: 3, 0x12: 2, 0x13: 3, 0x14: 3, 0x15: 2, 0x16: 2, 0x17: 2,
             0x18: 2, 0x19: 2, 0x36: 2, 0x37: 2, 0x38: 2, 0x39: 2, 0x3a: 2, 0x84: 3,
             0xa9: 2, 0xbc: 2, 0xc5: 4, 0xb2: 3, 0xb3: 3, 0xb4: 3, 0xb5: 3, 0xb6: 3,
             0xb7: 3, 0xb8: 3, 0xbb: 3, 0xbd: 3, 0xc0: 3, 0xc1: 3, 0xb9: 5, 0xba: 5,
             0xc8: 5, 0xc9: 5}
    fixed.update({op: 3 for op in range(0x99, 0xa9)})
    fixed.update({0xc6: 3, 0xc7: 3})
    bcis, pc = [], 0
    while pc < len(code):
        bcis.append(pc)
        op = code[pc]
        if op == 0xaa:
            aligned = (pc + 4) & ~3
            if aligned + 12 > len(code):
                raise ValueError(f'truncated tableswitch at {pc}')
            low = int.from_bytes(code[aligned + 4:aligned + 8], 'big', signed=True)
            high = int.from_bytes(code[aligned + 8:aligned + 12], 'big', signed=True)
            if high < low:
                raise ValueError(f'invalid tableswitch at {pc}')
            size = aligned - pc + 12 + 4 * (high - low + 1)
        elif op == 0xab:
            aligned = (pc + 4) & ~3
            if aligned + 8 > len(code):
                raise ValueError(f'truncated lookupswitch at {pc}')
            pairs = int.from_bytes(code[aligned + 4:aligned + 8], 'big', signed=True)
            if pairs < 0:
                raise ValueError(f'invalid lookupswitch at {pc}')
            size = aligned - pc + 8 + 8 * pairs
        elif op == 0xc4:
            if pc + 1 >= len(code):
                raise ValueError(f'truncated wide at {pc}')
            size = 6 if code[pc + 1] == 0x84 else 4
        else:
            size = fixed.get(op, 1)
        if pc + size > len(code):
            raise ValueError(f'instruction overruns Code at {pc}')
        pc += size
    return bcis


def parse_class_file(data: bytes) -> dict:
    """Small independent class-header/Code reader used to bind BCI facts to class bytes."""
    pos = 0

    def take(size: int) -> bytes:
        nonlocal pos
        value = data[pos:pos + size]
        if len(value) != size:
            raise ValueError('truncated class file')
        pos += size
        return value

    def u1() -> int:
        return take(1)[0]

    def u2() -> int:
        return int.from_bytes(take(2), 'big')

    def u4() -> int:
        return int.from_bytes(take(4), 'big')

    need(u4() == 0xCAFEBABE, 'bad classfile magic')
    minor, major = u2(), u2()
    cp_count = u2()
    cp = [None] * cp_count
    i = 1
    while i < cp_count:
        tag = u1()
        if tag == 1:
            cp[i] = ('utf8', take(u2()).decode('utf-8', errors='strict'))
        elif tag in (3, 4):
            take(4)
        elif tag in (5, 6):
            take(8)
            i += 1
        elif tag in (7, 8, 16, 19, 20):
            cp[i] = (tag, u2())
        elif tag in (9, 10, 11, 12, 17, 18):
            cp[i] = (tag, u2(), u2())
        elif tag == 15:
            take(3)
        else:
            raise ValueError(f'unknown constant-pool tag {tag}')
        i += 1

    def utf(index: int) -> str:
        entry = cp[index]
        need(entry is not None and entry[0] == 'utf8', f'bad UTF8 constant pool index {index}')
        return entry[1]

    def skip_attributes() -> list[tuple[str, bytes]]:
        attrs = []
        for _ in range(u2()):
            name, size = utf(u2()), u4()
            attrs.append((name, take(size)))
        return attrs

    access, this_class = u2(), u2()
    u2()  # super_class
    class_name = utf(cp[this_class][1])
    for _ in range(u2()):
        u2()
    fields = []
    for index in range(u2()):
        flags, name, descriptor = u2(), utf(u2()), utf(u2())
        attributes = skip_attributes()
        fields.append({'index': index, 'name': name, 'descriptor': descriptor,
                       'access': flags, 'attributes': [name for name, _ in attributes]})
    methods = []
    for index in range(u2()):
        flags, name, descriptor = u2(), utf(u2()), utf(u2())
        attrs = skip_attributes()
        code = None
        for attr_name, blob in attrs:
            if attr_name != 'Code':
                continue
            cursor = 0

            def read2() -> int:
                nonlocal cursor
                value = int.from_bytes(blob[cursor:cursor + 2], 'big')
                if len(blob[cursor:cursor + 2]) != 2:
                    raise ValueError('truncated Code attribute')
                cursor += 2
                return value

            def read4() -> int:
                nonlocal cursor
                chunk = blob[cursor:cursor + 4]
                if len(chunk) != 4:
                    raise ValueError('truncated Code attribute')
                cursor += 4
                return int.from_bytes(chunk, 'big')

            max_stack, max_locals, length = read2(), read2(), read4()
            code_bytes = blob[cursor:cursor + length]
            need(len(code_bytes) == length, 'truncated method Code bytes')
            cursor += length
            rows = []
            for _ in range(read2()):
                rows.append([read2(), read2(), read2(), read2()])
            for _ in range(read2()):
                read2(); size = read4(); cursor += size
                need(cursor <= len(blob), 'truncated nested Code attribute')
            need(cursor == len(blob), 'trailing Code attribute bytes')
            code = {'max_stack': max_stack, 'max_locals': max_locals, 'code_length': length,
                    'code_hex': code_bytes.hex(), 'rows': rows, 'bcis': decode_bcis(code_bytes)}
        methods.append({'index': index, 'name': name, 'descriptor': descriptor, 'access': flags, 'code': code})
    skip_attributes()
    need(pos == len(data), 'trailing classfile bytes')
    return {'minor': minor, 'major': major, 'access': access, 'class_name': class_name,
            'fields': fields, 'methods': methods}


def source_closure(root: Path) -> dict[str, dict]:
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
    result = {}
    for path in sorted(set(candidates)):
        if path.is_file() and not path.is_symlink():
            result[path.relative_to(root).as_posix()] = file_pin(path)
    return result


def validate_source_and_dependencies(execution: dict, out: Path, root: Path) -> dict:
    need(execution.get('source_unchanged') is True, 'collector did not confirm source unchanged')
    need(execution.get('target_absent') is True and not (root / 'target').exists(), 'target must be absent at review time')
    before = execution.get('source_pins_before')
    need(isinstance(before, dict) and before, 'missing root-v3 source_pins_before inventory')
    current = source_closure(root)
    need(before == current, 'current workspace source closure differs from capture source_pins_before')

    source = verify_pin(execution['source'], 'upstream complete source')
    source_copy = verify_pin(execution['source_copy'], 'copied upstream source')
    need(Path(execution['source']['path']) == SOURCE and execution['source']['sha256'] == SOURCE_SHA,
         'upstream source path/SHA differs from the pinned TestNestedLoops5.java')
    need(source_copy == source and Path(execution['source_copy']['path']) == out / 'source' / SOURCE.name,
         'copied complete upstream source is not byte-identical or at the collector destination')
    need((out / 'source' / SOURCE.name).read_bytes() == source, 'source copy path does not contain original source bytes')

    driver = verify_pin(execution['driver'], 'copied driver')
    driver_source = (PREP / 'driver' / 'TestNestedLoops5Driver.java').read_bytes()
    need(driver == driver_source and Path(execution['driver']['path']) == out / 'driver-source' / 'TestNestedLoops5Driver.java'
         and (out / 'driver-source' / 'TestNestedLoops5Driver.java').read_bytes() == driver,
         'copied standalone reflection driver differs from reviewed source')
    runtime = verify_pin(execution['runtime_bound'], 'copied CodeSource runtime guard')
    need(runtime == RUNTIME_BOUND.read_bytes() and Path(execution['runtime_bound']['path']) == out / RUNTIME_BOUND.name,
         'copied runtime guard differs from its pinned original or collector destination')
    need((out / RUNTIME_BOUND.name).read_bytes() == runtime, 'runtime guard copy has unexpected path/content')

    dep_record = execution['dependency_reference']
    dep_bytes = verify_pin(dep_record, 'NL3 dependency reference')
    need(Path(dep_record['path']) == DEPENDENCY_REFERENCE and sha(dep_bytes) == DEPENDENCY_REFERENCE_SHA,
         'dependency reference path/SHA differs from the pinned prior receipt')
    dep = json.loads(dep_bytes)
    need(execution.get('classpath') == dep.get('classpath'), 'classpath differs from pinned dependency receipt')
    need(execution.get('harness_jar') == dep.get('harness_jar'), 'harness jar record differs from dependency receipt')
    dep_dirs = dep.get('directory_pins', dep.get('directory_pins_before'))
    need(dep_dirs and execution.get('directory_pins') == dep_dirs, 'build class directory pin tree differs from dependency receipt')
    need(execution.get('jar_pins') == dep.get('jar_pins'), 'dependency jar records differ from dependency receipt')

    harness = verify_pin(execution['harness_jar'], 'JADX harness jar')
    with zipfile.ZipFile(Path(execution['harness_jar']['path'])) as jar:
        need(not any('TestNestedLoops5' in name for name in jar.namelist()), 'dependency harness already contains the target class')
    jar_rows = [execution['harness_jar'], *execution['jar_pins']]
    for row in jar_rows:
        verify_pin(row, 'dependency JAR')
    directory_count = 0
    for folder, expected in dep_dirs.items():
        actual = {}
        base = Path(folder)
        for path in sorted(base.rglob('*.class')):
            actual[path.relative_to(base).as_posix()] = file_pin(path)
        need(actual == expected, f'JADX build-class directory differs from dependency receipt: {folder}')
        directory_count += len(actual)

    manifest_bytes = JDK_MANIFEST.read_bytes()
    need(sha(manifest_bytes) == JDK_MANIFEST_SHA and execution['jdk_manifest'].get('sha256') == JDK_MANIFEST_SHA,
         'JDK manifest pin mismatch')
    need(Path(execution['jdk_manifest']['path']) == JDK_MANIFEST.resolve(), 'JDK manifest path mismatch')
    manifest = json.loads(manifest_bytes)
    for leg in RUNTIMES:
        item = next(row for row in manifest['legs'] if row['leg'] == leg)
        recorded = execution['jdk_manifest']['tools'][leg]
        for tool in ('java', 'javac', 'javap'):
            expected = item['jdk_tools'][tool]
            actual = recorded[tool]
            need(Path(expected['path']).resolve() == Path(actual['path']).resolve(), f'{leg}/{tool} JDK tool path differs')
            verify_pin(actual, f'{leg} {tool}')
            need(actual['sha256'] == expected['sha256'], f'{leg}/{tool} JDK tool digest differs from manifest')

    static_pins = (
        ('cli', CLI, CLI_SHA), ('jadx', JADX, JADX_SHA), ('jadx_jar', JADX_JAR, JADX_JAR_SHA),
        ('guard', GUARD, GUARD_SHA), ('scanner', SCANNER, SCANNER_SHA),
    )
    for key, path, digest in static_pins:
        record = execution[key]
        verify_pin(record, key)
        need(Path(record['path']).resolve() == path.resolve() and record['sha256'] == digest,
             f'{key} path/digest differs from frozen product pin')
    collector_copy = out / 'collector.py'
    need(collector_copy.is_file() and collector_copy.read_bytes() == (PREP / 'collect_baseline.py').read_bytes(),
         'captured collector.py is not the root-v3 collector; do not use the older Luna source-pins.json')

    return {'source_closure_files': len(current), 'source_closure_equal': True,
            'source_copy_byte_identical': True, 'driver_copy_byte_identical': True,
            'runtime_bound_copy_byte_identical': True, 'dependency_jars': len(jar_rows),
            'dependency_class_files': directory_count, 'dependency_harness_excludes_target': True,
            'jdk_legs': list(RUNTIMES), 'frozen_cli_and_jadx_pins_checked': True,
            'authority': 'execution.json and its copied root-v3 collector; ignored prep source-pins.json from old Luna draft'}


def validate_commands(execution: dict, out: Path) -> tuple[dict[str, dict], dict]:
    rows = execution.get('commands')
    need(isinstance(rows, list) and rows, 'missing command inventory')
    by_index, by_label, raw_count = {}, {}, 0
    for row in rows:
        index, label = row.get('index'), row.get('label')
        need(isinstance(index, int) and index not in by_index, f'duplicate/invalid command index: {index}')
        need(isinstance(label, str) and label not in by_label, f'duplicate/invalid command label: {label}')
        need(row.get('guard_stop') is None, f'guard stopped command {label}: {row.get("guard_stop")}')
        need(Path(row.get('cwd', '')).resolve() == ROOT.resolve(), f'command ran from unexpected cwd: {label}')
        streams = row.get('streams')
        need(isinstance(streams, dict) and set(streams) == {'stdout', 'stderr'}, f'{label}: stdout/stderr pins are incomplete')
        for stream_name, stream in streams.items():
            data = verify_pin(stream, f'command {index} {stream_name}')
            need(Path(stream['path']).parent == out / 'raw', f'command {index}: raw stream is outside output raw directory')
            need(Path(stream['path']) == out / 'raw' / f'{index}.{stream_name}.raw', f'command {index}: raw stream path does not match collector naming')
            raw_count += 1
        by_index[index], by_label[label] = row, row
    need(sorted(by_index) == list(range(1, len(rows) + 1)), 'command indexes are not a complete 1-based sequence')
    expected_labels = {
        'original-javac8-harness-major-boundary', 'compile-independent-reflection-driver',
        'driver-code-source-wrong-root-control', 'jadx-version',
    }
    expected_labels |= {f'original-compile-{p}' for p in PROFILES}
    expected_labels |= {f'original-javap-{p}' for p in PROFILES}
    expected_labels |= {f'original-runtime-{p}-{r}' for p in PROFILES for r in RUNTIMES}
    expected_labels |= {f'original-check-runtime-{p}-{r}' for p in PROFILES for r in RUNTIMES}
    expected_labels |= {f'jarde-{p}-{form}-{e}' for p in PROFILES for form in FORMS for e in EVIDENCE}
    expected_labels |= {f'jadx-{p}-{form}' for p in PROFILES for form in ('child', 'bundle')}
    need(set(by_label) == expected_labels and len(rows) == 28, 'command label inventory is not the complete 28-command baseline matrix')
    need(raw_count == 56, f'expected 56 pinned raw command streams, got {raw_count}')
    return by_label, {'command_count': len(rows), 'raw_stream_count': raw_count, 'all_guard_stops_clear': True}


def check_command(row: dict, expected_argv: list[str], exit_code: int | None = None) -> None:
    need(row.get('argv') == expected_argv, f"{row.get('label')}: argv differs from expected operation")
    if exit_code is not None:
        need(row.get('exit_code') == exit_code, f"{row.get('label')}: expected exit {exit_code}, got {row.get('exit_code')}")


def validate_profiles(execution: dict, by_label: dict, out: Path) -> tuple[dict, dict]:
    manifest = json.loads(JDK_MANIFEST.read_text(encoding='utf-8'))
    javac23 = str(Path(next(row['jdk_tools']['javac']['path'] for row in manifest['legs'] if row['leg'] == 'javac23')).resolve())
    javap23 = str(Path(next(row['jdk_tools']['javap']['path'] for row in manifest['legs'] if row['leg'] == 'javac23')).resolve())
    profiles = execution.get('profiles', [])
    need(len(profiles) == 2 and {row.get('profile') for row in profiles} == set(PROFILES), 'debug/no-debug compiler profiles incomplete')
    fact_profiles, classes, compile_summary = {}, {}, {}
    for profile in profiles:
        name = profile['profile']
        target = profile['target']
        outer = profile['outer']
        need(target['major'] == 52 and target['class_name'] == INTERNAL_CLASS, f'{name}: target class identity/major mismatch')
        need(outer['major'] == 52 and outer['class_name'] == OUTER_CLASS, f'{name}: outer class identity/major mismatch')
        observed = [(m['name'], m['descriptor'], m['access']) for m in target['methods']]
        need(observed == METHODS, f'{name}: expected the three complete physical methods in source order, got {observed}')
        need(profile.get('method_code_facts') == target['methods'], f'{name}: method_code_facts differ from parsed target class facts')
        target_pin = verify_pin({'path': target['path'], 'bytes': target['bytes'], 'sha256': target['sha256']}, f'{name} target class')
        outer_pin = verify_pin({'path': outer['path'], 'bytes': outer['bytes'], 'sha256': outer['sha256']}, f'{name} outer class')
        target_parsed = parse_class_file(target_pin)
        outer_parsed = parse_class_file(outer_pin)
        need(target_parsed['major'] == target['major'] and target_parsed['class_name'] == target['class_name']
             and target_parsed['methods'] == target['methods'], f'{name}: receipt Code facts do not parse from pinned target class bytes')
        need(outer_parsed['major'] == outer['major'] and outer_parsed['class_name'] == outer['class_name'],
             f'{name}: outer class identity/major do not parse from pinned class bytes')
        class_files = profile.get('class_files', [])
        need(len(class_files) == 2 and {row['sha256'] for row in class_files} == {target['sha256'], outer['sha256']}, f'{name}: class output pins incomplete')
        for row in class_files:
            verify_pin(row, f'{name} compiled class output')
        class_root = Path(profile['classes'])
        discovered = sorted(path.relative_to(class_root).as_posix() for path in class_root.rglob('*.class'))
        expected_class_paths = sorted((Path(INTERNAL_CLASS + '.class'), Path(OUTER_CLASS + '.class')))
        need(discovered == [path.as_posix() for path in expected_class_paths], f'{name}: complete source did not produce exactly outer+target classes')
        need(Path(target['path']) == class_root / (INTERNAL_CLASS + '.class')
             and Path(outer['path']) == class_root / (OUTER_CLASS + '.class'), f'{name}: parsed class paths differ from clean compile root')
        for method in target['methods']:
            code = method.get('code')
            need(isinstance(code, dict), f'{name}: missing Code facts for {method["name"]}{method["descriptor"]}')
            bcis = bci_set(method)
            # Validate the retained BCI census is consistent with the complete encoded Code bytes.
            need(bcis and max(bcis) < code['code_length'], f'{name}: invalid physical BCI range')
        classes[name] = {'target': target, 'target_bytes': target_pin, 'outer': outer, 'outer_bytes': outer_pin}
        fact_profiles[name] = target['methods']

        compile = by_label[f'original-compile-{name}']
        need(compile.get('exit_code') == 0, f'{name}: original full source compile failed')
        flag = '-g' if name == 'debug' else '-g:none'
        expected_compile = [javac23, '--release', '8', flag, '-encoding', 'UTF-8', '-proc:none',
                            '-classpath', os.pathsep.join(execution['classpath']), '-sourcepath', str(out / 'empty-sourcepath'),
                            '-d', str(class_root), execution['source_copy']['path']]
        need(compile['argv'] == expected_compile, f'{name}: source compile argv differs from pinned whole-source compile')
        javap = by_label[f'original-javap-{name}']
        need(javap.get('exit_code') == 0 and javap['argv'] == [javap23, '-p', '-s', '-c', '-v', target['path']],
             f'{name}: javap fact capture failed or used a different target')
        compile_summary[name] = {'target_bytes': len(target_pin), 'outer_bytes': len(outer_pin),
                                 'method_code': [{'name': m['name'], 'descriptor': m['descriptor'],
                                                 'code_length': m['code']['code_length'],
                                                 'instruction_count': len(m['code']['bcis']),
                                                 'bcis': m['code']['bcis'], 'exception_rows': m['code']['rows']}
                                                for m in target['methods']]}
    debug, no_debug = fact_profiles['debug'], fact_profiles['no-debug']
    need([(m['name'], m['descriptor'], m['access'], m['code']) for m in debug]
         == [(m['name'], m['descriptor'], m['access'], m['code']) for m in no_debug],
         'debug/no-debug Code facts differ for one or more of the three physical methods')
    return classes, {'profiles': compile_summary, 'debug_no_debug_all_three_Code_methods_identical': True}


def validate_runtime(execution: dict, by_label: dict, out: Path, classes: dict) -> dict:
    manifest = json.loads(JDK_MANIFEST.read_text(encoding='utf-8'))
    jdk_tools = {row['leg']: {key: str(Path(item['path']).resolve()) for key, item in row['jdk_tools'].items()}
                 for row in manifest['legs'] if row['leg'] in RUNTIMES}
    boundary = by_label['original-javac8-harness-major-boundary']
    need(boundary['exit_code'] != 0, 'javac8 harness boundary unexpectedly succeeded')
    stderr = verify_pin(boundary['streams']['stderr'], 'javac8 major-55 boundary stderr')
    need(b'55.0' in stderr and execution.get('javac8_boundary', {}).get('observed_major_55_diagnostic') is True,
         'javac8 compile boundary did not retain the major-55 diagnostic')
    expected_boundary = [jdk_tools['javac8']['javac'], '-source', '8', '-target', '8', '-g', '-encoding', 'UTF-8',
                         '-proc:none', '-classpath', os.pathsep.join(execution['classpath']),
                         '-sourcepath', str(out / 'empty-sourcepath'), '-d', str(out / 'javac8-boundary/classes'),
                         execution['source_copy']['path']]
    need(boundary['argv'] == expected_boundary, 'boundary command is not the captured javac8 source/target attempt')

    driver_compile = by_label['compile-independent-reflection-driver']
    need(driver_compile['exit_code'] == 0 and driver_compile['argv'][0] == jdk_tools['javac8']['javac'], 'independent reflection driver compile failed')
    empty = str(out / 'empty-sourcepath')
    expected_driver_compile = [jdk_tools['javac8']['javac'], '-source', '8', '-target', '8', '-g', '-encoding', 'UTF-8',
                               '-proc:none', '-classpath', empty, '-sourcepath', empty, '-d', str(out / 'driver-classes'),
                               str(out / 'driver-source/TestNestedLoops5Driver.java')]
    need(driver_compile['argv'] == expected_driver_compile, 'independent driver was not compiled from its pinned standalone source')
    need(execution.get('driver_classes'), 'driver class outputs are absent')
    for row in execution['driver_classes']:
        verify_pin(row, 'compiled reflection driver class')

    oracle_bytes = verify_pin(execution['original_oracle'], 'observed four-run core oracle')
    need(Path(execution['original_oracle']['path']) == out / 'original-oracle.stdout', 'core oracle path differs from collector destination')
    oracle_text = oracle_bytes.decode('utf-8', errors='strict')
    lines = oracle_text.splitlines()
    need(len(lines) == 3 and all(line.startswith('CASE ') for line in lines), 'core oracle must contain its three observed CASE lines')
    need(oracle_text == execution.get('observed_oracle_stdout'), 'stored observed core oracle differs from pinned bytes')
    original_runtime = execution.get('original_runtime', [])
    need(len(original_runtime) == 4, 'expected four core runtime observations')
    seen = set()
    for cell in original_runtime:
        key = (cell['profile'], cell['runtime'])
        need(key not in seen and key[0] in PROFILES and key[1] in RUNTIMES, f'duplicate/unexpected core runtime cell: {key}')
        seen.add(key)
        row = by_label[f'original-runtime-{key[0]}-{key[1]}']
        need(cell.get('command') == row['index'] and cell.get('exit_code') == 0 == row['exit_code'], f'{key}: original runtime failed')
        observed = verify_pin(row['streams']['stdout'], f'{key} core stdout')
        need(observed == oracle_bytes and cell.get('stdout_sha256') == sha(observed), f'{key}: core output differs from observed three-case oracle')
        cp = os.pathsep.join(cell['classpath'])
        expected_profile_root = str(Path(classes[key[0]]['target']['path']).parents[4])
        expected_cp = os.pathsep.join([expected_profile_root,
                                       str(out / 'driver-classes'),
                                       next(item['path'] for item in execution['jar_pins'] if Path(item['path']).name.startswith('assertj-core-'))])
        need(cp == expected_cp, f'{key}: core runtime classpath is not the profile + driver + AssertJ roots')
        argv = row['argv']
        need(argv[-3:] == ['TestNestedLoops5Driver', expected_profile_root, 'core'], f'{key}: driver target root/mode mismatch')
        need(argv == [sys.executable, '-B', execution['runtime_bound']['path'], '30', jdk_tools[key[1]]['java'],
                      '-Xverify:all', '-cp', cp, 'TestNestedLoops5Driver', expected_profile_root, 'core'],
             f'{key}: runtime command did not use the captured verifier and JVM verification')
    need(seen == {(p, r) for p in PROFILES for r in RUNTIMES}, 'core runtime matrix omits a profile/JVM pair')

    check_rows = execution.get('original_check_runtime', [])
    need(len(check_rows) == 4, 'expected two profile × two JVM check attempts')
    check_outputs, check_seen = [], set()
    for cell in check_rows:
        key = (cell['profile'], cell['runtime'])
        need(key not in check_seen and key[0] in PROFILES and key[1] in RUNTIMES, f'duplicate/unexpected check runtime cell: {key}')
        check_seen.add(key)
        row = by_label[f'original-check-runtime-{key[0]}-{key[1]}']
        need(cell.get('command') == row['index'] and cell.get('exit_code') == row['exit_code'], f'{key}: check runtime command/result mismatch')
        err = verify_pin(row['streams']['stderr'], f'{key} check stderr')
        stdout = verify_pin(row['streams']['stdout'], f'{key} check stdout')
        expected_profile_root = str(Path(classes[key[0]]['target']['path']).parents[4])
        expected_classpath = os.pathsep.join([expected_profile_root, str(out / 'driver-classes'), *execution['classpath']])
        need(row['argv'] == [sys.executable, '-B', execution['runtime_bound']['path'], '30', jdk_tools[key[1]]['java'],
                             '-Xverify:all', '-cp', expected_classpath, 'TestNestedLoops5Driver', expected_profile_root, 'check'],
             f'{key}: check runtime did not use pinned classpath/wrapper/JVM verification')
        if key[1] == 'javac8':
            need(row['exit_code'] != 0 and b'UnsupportedClassVersionError' in err and b'55.0' in err,
                 f'{key}: expected recorded major-55 harness boundary')
            need(cell.get('status') == 'harness-major55-boundary-no-check-result', f'{key}: wrong boundary classification')
        else:
            need(row['exit_code'] == 0 and cell.get('status') == 'observed-check-oracle', f'{key}: JVM23 check case failed')
            check_outputs.append(stdout)
    need(check_seen == {(p, r) for p in PROFILES for r in RUNTIMES}, 'check runtime matrix omits a profile/JVM pair')
    check_oracle = verify_pin(execution['original_check_oracle'], 'JVM23 check oracle')
    need(Path(execution['original_check_oracle']['path']) == out / 'original-check-oracle.stdout', 'check oracle path differs from collector destination')
    need(len(check_outputs) == 2 and check_outputs[0] == check_outputs[1] == check_oracle,
         'two JVM23 check results do not agree with captured check oracle')

    wrong = execution.get('wrong_root_control', {})
    wrong_row = by_label['driver-code-source-wrong-root-control']
    wrong_stderr = verify_pin(wrong_row['streams']['stderr'], 'wrong-root CodeSource refusal')
    assertj_path = next(item['path'] for item in execution['jar_pins'] if Path(item['path']).name.startswith('assertj-core-'))
    wrong_cp = os.pathsep.join([str(Path(classes['debug']['target']['path']).parents[4]),
                                str(out / 'driver-classes'), assertj_path])
    expected_wrong = [sys.executable, '-B', execution['runtime_bound']['path'], '30', jdk_tools['javac23']['java'],
                      '-Xverify:all', '-cp', wrong_cp, 'TestNestedLoops5Driver', str(out / 'empty-sourcepath'), 'core']
    need(wrong_row['argv'] == expected_wrong, 'wrong-root control argv is not bound to the intentional wrong expected root')
    need(wrong.get('command') == wrong_row['index'] and wrong.get('exit_code') == wrong_row['exit_code'] != 0
         and b'target loaded from' in wrong_stderr, 'wrong-root CodeSource control did not reject the expected location')
    version = by_label['jadx-version']
    need(version['argv'] == [str(JADX), '--version'], 'JADX version command argv differs')
    version_text = (verify_pin(version['streams']['stdout'], 'JADX version stdout')
                    + verify_pin(version['streams']['stderr'], 'JADX version stderr')).decode('utf-8', errors='replace')
    need(version['exit_code'] == 0 and '1.5.6' in version_text, 'JADX version command did not identify 1.5.6')
    return {'javac8_major55_compile_boundary': True, 'core_oracle_case_count': len(lines),
            'core_runtime_cells': len(original_runtime), 'core_runtime_all_match_oracle': True,
            'JVM23_check_cells': len(check_outputs), 'JVM8_check_major55_boundaries': 2,
            'wrong_root_control_rejected': True, 'JADX_version': '1.5.6'}


def map_method(method: dict, fact: dict, owner: dict, label: str) -> dict:
    body = method.get('outcome', {}).get('report')
    need(isinstance(body, dict), f'{label}: method report missing')
    text = body.get('text')
    need(isinstance(text, str), f'{label}: method report text missing')
    encoded = text.encode('utf-8')
    boundaries = byte_boundaries(encoded)
    segments = body.get('source_map', {}).get('segments')
    need(isinstance(segments, list), f'{label}: source map segment list missing')
    physical = bci_set(fact)
    mapped, projected = set(), []
    item_identity = method['item']['identity']
    for seg in segments:
        start, end = seg.get('start'), seg.get('end')
        need(isinstance(start, int) and isinstance(end, int) and 0 <= start <= end <= len(encoded), f'{label}: map byte range outside method UTF-8 text')
        need(start in boundaries and end in boundaries, f'{label}: map endpoint splits a UTF-8 code point')
        origin = seg.get('origin')
        need(isinstance(origin, dict) and isinstance(origin.get('primary'), dict), f'{label}: map segment lacks primary origin')
        origins = [origin['primary'], *origin.get('derived', [])]
        out_origins = []
        for record in origins:
            bci = record.get('bci')
            need(isinstance(bci, int) and bci in physical, f'{label}: source origin BCI is not in this physical method: {bci}')
            method_id = record.get('method')
            need(isinstance(method_id, dict), f'{label}: source origin has no physical method id')
            need(method_id.get('owner') == owner, f'{label}: source origin owner differs from current physical class input')
            need(method_id.get('name') == item_identity.get('name') and method_id.get('descriptor') == item_identity.get('descriptor'),
                 f'{label}: source origin points at a different physical method')
            mapped.add(bci)
            out_origins.append({'bci': bci, 'provenance': record.get('provenance'),
                                'method_name': method_id.get('name'), 'method_descriptor': method_id.get('descriptor')})
        projected.append({'start': start, 'end': end, 'origins': out_origins})
    missing, extra = sorted(physical - mapped), sorted(mapped - physical)
    quality = body.get('quality')
    need(isinstance(quality, str), f'{label}: method quality absent')
    if quality == 'structured':
        need(not missing and not extra, f'{label}: structured report misses physical BCIs: {missing}')
    return {'quality': quality, 'fallbacks': body.get('fallbacks'), 'content': body.get('content'),
            'declaration': method.get('declaration'), 'method_text': method.get('text'),
            'body_text': text, 'source_map': projected,
            'physical_bcis': sorted(physical), 'mapped_bcis': sorted(mapped),
            'missing_bcis': missing, 'unexpected_bcis': extra,
            'complete_physical_bci_coverage': not missing and not extra}


def identity_bytes(row: dict, key: str) -> bytes:
    value = row.get(key)
    need(isinstance(value, list) and all(isinstance(byte, int) and 0 <= byte <= 255 for byte in value), f'malformed physical identity {key}')
    return bytes(value)


def validate_jarde(execution: dict, by_label: dict, classes: dict, out: Path) -> dict:
    cells = execution.get('jarde_cells', [])
    expected = {(p, f, e) for p in PROFILES for f in FORMS for e in EVIDENCE}
    keys = {(c.get('profile'), c.get('input_form'), c.get('evidence')) for c in cells}
    need(len(cells) == 8 and keys == expected, 'Jarde class-source matrix is not the exact 2×2×2 inventory')
    projections, owners, cell_rows = {}, {}, []
    complete = True
    for cell in cells:
        key = (cell['profile'], cell['input_form'], cell['evidence'])
        label = f'jarde-{key[0]}-{key[1]}-{key[2]}'
        command = by_label[label]
        need(cell.get('command') == command['index'] and cell.get('exit_code') == command['exit_code'], f'{key}: command row mismatch')
        input_pin = verify_pin(cell['input'], f'{key} Jarde input')
        profile = classes[key[0]]
        target_path = Path(profile['target']['path'])
        if key[1] == 'child':
            need(Path(cell['input']['path']) == target_path and input_pin == profile['target_bytes'], f'{key}: standalone input does not bind this profile class')
            policy = 'single-class'
        else:
            bundle_path = Path(cell['input']['path'])
            need(bundle_path == out / (key[0] + '.jar'), f'{key}: archive-child input path differs from collector bundle')
            need(bundle_path.suffix == '.jar', f'{key}: archive-child input is not a JAR')
            with zipfile.ZipFile(bundle_path) as jar:
                entries = set(jar.namelist())
                need(entries == {INTERNAL_CLASS + '.class', OUTER_CLASS + '.class'}, f'{key}: bundle class inventory differs')
                need(jar.read(INTERNAL_CLASS + '.class') == profile['target_bytes'], f'{key}: archive target bytes differ from profile class')
                need(jar.read(OUTER_CLASS + '.class') == profile['outer_bytes'], f'{key}: archive outer bytes differ from profile class')
            policy = 'plain-jar'
        argv = [str(CLI), 'class-source', '--input', str(cell['input']['path']), '--class', INTERNAL_CLASS,
                '--policy', policy, '--release', '8', '--format', 'json']
        if key[2] == 'all':
            argv += ['--evidence', 'all']
        check_command(command, argv)
        report_bytes = verify_pin(cell['report'], f'{key} class-source JSON report')
        need(report_bytes == verify_pin(command['streams']['stdout'], f'{key} command stdout'), f'{key}: report differs from raw stdout')
        need(Path(cell['report']['path']) == out / 'jarde' / key[0] / key[1] / key[2] / 'report.json', f'{key}: report is not at collector destination')
        row = {'cell': list(key), 'exit_code': command['exit_code'], 'report_bytes': len(report_bytes), 'report_sha256': sha(report_bytes)}
        try:
            document = json.loads(report_bytes.decode('utf-8', errors='strict'))
        except (UnicodeDecodeError, json.JSONDecodeError) as error:
            row['json_parse_error'] = str(error)
            row['method_checks_available'] = False
            complete = False
            cell_rows.append(row)
            continue
        if cell.get('json_parse_error'):
            need(False, f'{key}: receipt claims JSON parse error despite a valid captured report')
        need(document.get('class') is not None and isinstance(document.get('methods'), list), f'{key}: class-source report lacks class/method records')
        owner = document['class']
        class_bytes = owner.get('class_bytes', {})
        need(class_bytes.get('length') == profile['target']['bytes'] and isinstance(class_bytes.get('digest'), str)
             and len(class_bytes['digest']) == 64, f'{key}: report class identity/length does not bind this method input')
        location = owner.get('location', {})
        expected_kind = 'standalone_root' if key[1] == 'child' else 'archive_entry'
        need(location.get('kind') == expected_kind, f'{key}: physical owner kind differs from input form')
        if key[1] == 'child':
            need(location.get('snapshot') == class_bytes['digest'], f'{key}: standalone snapshot differs from class digest')
        else:
            entry = location.get('entry', {})
            raw_name = identity_bytes(entry, 'raw_name').decode('utf-8', errors='strict')
            need(raw_name == INTERNAL_CLASS + '.class', f'{key}: archive entry owner path mismatch: {raw_name}')
            snapshot = entry.get('origin', {}).get('snapshot')
            need(isinstance(snapshot, str) and len(snapshot) == 64, f'{key}: archive owner has no root snapshot identity')
        owners[key] = {'owner': owner, 'class_digest': class_bytes['digest'], 'kind': expected_kind}
        methods = document['methods']
        need(len(methods) == len(METHODS), f'{key}: report omitted one or more physical methods')
        method_projections, rows, identity_rows = {}, [], []
        facts = profile['target']['methods']
        for index, (method, fact, expected_method) in enumerate(zip(methods, facts, METHODS)):
            name, descriptor, access = expected_method
            item = method['item']
            got_name = bytes(item['name']['raw']).decode('utf-8', errors='strict')
            got_desc = bytes(item['descriptor']['raw']).decode('utf-8', errors='strict')
            identity = item['identity']
            need((got_name, got_desc, item['access_flags'], item['index']) == (name, descriptor, access, index),
                 f'{key}: physical declaration/order/access differs at index {index}')
            need(identity_bytes(identity, 'name').decode('utf-8', errors='strict') == name
                 and identity_bytes(identity, 'descriptor').decode('utf-8', errors='strict') == descriptor,
                 f'{key}: physical method identity bytes differ from declaration')
            need(identity.get('owner') == owner, f"{key}: method owner differs from this report's physical class")
            nested = method['outcome']['report']
            binding = nested.get('artifact', {}).get('binding', {})
            need(binding.get('member_ordinal') == index and binding.get('text_bytes') == len(nested.get('text', '').encode('utf-8')), f'{key}: binding ordinal/text bytes mismatch')
            artifact_method = nested.get('artifact', {}).get('binding', {}).get('method')
            need(artifact_method == identity, f'{key}: report artifact method differs from physical declaration')
            need((fact['name'], fact['descriptor'], fact['access']) == (name, descriptor, access), f'{key}: declaration differs from same-profile class Code facts')
            mapped = map_method(method, fact, owner, f'{key}/{name}{descriptor}')
            method_projections[name + descriptor] = mapped
            rows.append({'name': name, 'descriptor': descriptor, **mapped})
            identity_rows.append({'index': index, 'name': name, 'descriptor': descriptor,
                                  'access': access, 'owner_kind': expected_kind})
        need(cell.get('method_inventory_matches') == ([(r['name'], r['descriptor']) for r in rows] == [(n, d) for n, d, _ in METHODS]),
             f'{key}: collector method inventory boolean is inconsistent')

        source_bytes = document.get('text', '').encode('utf-8')
        if cell.get('whole_source'):
            source_saved = verify_pin(cell['whole_source'], f'{key} complete class source')
            need(source_saved == source_bytes, f'{key}: saved whole-source differs from report text')
            need(Path(cell['whole_source']['path']) == out / 'jarde' / key[0] / key[1] / key[2] / 'whole-source.java', f'{key}: whole source path differs from collector destination')
        else:
            row['whole_source_missing'] = True
            complete = False
        projections[key] = {'text': document.get('text'), 'methods': method_projections}
        row.update({'method_checks_available': True, 'method_inventory': identity_rows,
                    'method_outcomes': [{'name': m['name'], 'descriptor': m['descriptor'],
                                         'quality': m['quality'], 'fallbacks': m['fallbacks'],
                                         'physical_bci_count': len(m['physical_bcis']),
                                         'mapped_bci_count': len(m['mapped_bcis']),
                                         'missing_bcis': m['missing_bcis'],
                                         'unexpected_bcis': m['unexpected_bcis'],
                                         'complete_physical_bci_coverage': m['complete_physical_bci_coverage']}
                                        for m in rows]})
        cell_rows.append(row)
    # Matching BLAKE3 class identity across the child and bundle input forms is an intra-profile check;
    # the owner location/snapshot identities remain input-specific.
    for profile in PROFILES:
        for evidence in EVIDENCE:
            child = owners.get((profile, 'child', evidence))
            bundle = owners.get((profile, 'bundle-child', evidence))
            if child and bundle:
                need(child['class_digest'] == bundle['class_digest'], f'{profile}/{evidence}: same class bytes have different report class digest')

    default_all, parity_complete = [], True
    for profile in PROFILES:
        for form in FORMS:
            low_key, full_key = (profile, form, 'default'), (profile, form, 'all')
            low, full = projections.get(low_key), projections.get(full_key)
            if low is None or full is None:
                default_all.append({'profile': profile, 'form': form, 'comparable': False})
                parity_complete = False
                continue
            stable = low == full
            need(stable, f'{profile}/{form}: default/all class text, method quality/fallback, declaration or source map differs')
            default_all.append({'profile': profile, 'form': form, 'comparable': True, 'stable': stable})
    return {'cell_count': len(cells), 'cells': cell_rows, 'default_all_parity': default_all,
            'all_method_reports_available': complete, 'default_all_comparison_complete': parity_complete,
            'same_class_digest_child_vs_bundle_by_profile': True,
            'fallback_maps_report_missing_BCIs_without_claiming_complete_coverage': True}


def validate_jadx(execution: dict, by_label: dict, classes: dict, out: Path) -> dict:
    cells = execution.get('jadx_cells', [])
    expected = {(p, form) for p in PROFILES for form in ('child', 'bundle')}
    seen = {(row.get('profile'), row.get('input_form')) for row in cells}
    need(len(cells) == 4 and seen == expected, 'JADX extraction inventory is not exactly two profiles × child/bundle')
    results = []
    complete = True
    for cell in cells:
        key = (cell['profile'], cell['input_form'])
        row = by_label[f'jadx-{key[0]}-{key[1]}']
        need(cell.get('command') == row['index'] and cell.get('exit_code') == row['exit_code'], f'{key}: extraction command/result mismatch')
        expected_input = classes[key[0]]['target']['path'] if key[1] == 'child' else str(out / (key[0] + '.jar'))
        dest = out / 'jadx' / key[0] / key[1]
        expected_argv = [str(JADX), '--config', 'none', '--threads-count', '1', '--no-res',
                         '--rename-flags', 'none', '-d', str(dest), expected_input]
        need(row['argv'] == expected_argv, f'{key}: JADX extraction flags/input differ from pinned invocation')
        sources = cell.get('sources', [])
        observed_java = sorted(path for path in dest.rglob('*.java')) if dest.exists() else []
        need([Path(source['path']).resolve() for source in sources] == [path.resolve() for path in observed_java],
             f'{key}: recorded JADX source inventory differs from files actually emitted')
        for source in sources:
            data = verify_pin(source, f'{key} emitted JADX source')
            need(Path(source['path']).suffix == '.java', f'{key}: source pin is not Java text')
            data.decode('utf-8', errors='strict')
        results.append({'profile': key[0], 'form': key[1], 'exit_code': row['exit_code'],
                        'emitted_java_files': len(sources),
                        'source_pins': [{'bytes': s['bytes'], 'sha256': s['sha256']} for s in sources]})
        if row['exit_code'] != 0 or not sources:
            complete = False
    return {'cell_count': len(cells), 'cells': results,
            'all_extractions_exit_zero_and_emitted_source': complete,
            'success_not_required_for_inventory_review': True}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('execution_json', type=Path)
    parser.add_argument('review_json', type=Path)
    parser.add_argument('--root', type=Path, default=ROOT)
    args = parser.parse_args()
    execution_path = args.execution_json.resolve(strict=True)
    output_path = args.review_json.resolve()
    need(not output_path.exists(), f'refusing to overwrite reviewer output: {output_path}')
    root = args.root.resolve(strict=True)
    need(root == ROOT.resolve(), f'unexpected workspace root: {root}')
    execution_raw = execution_path.read_bytes()
    execution = json.loads(execution_raw)
    need(execution.get('schema') == 'cf11-original-testnestedloops5-baseline-v1', 'unexpected execution receipt schema')
    need(execution.get('status') == 'captured-pending-independent-review', 'receipt is not terminal captured-pending-review')
    need(execution.get('finished_utc') and execution.get('error') is None, 'receipt lacks terminal finish or records a collector error')

    source_checks = validate_source_and_dependencies(execution, execution_path.parent, root)
    by_label, raw_summary = validate_commands(execution, execution_path.parent)
    classes, class_summary = validate_profiles(execution, by_label, execution_path.parent)
    runtime_summary = validate_runtime(execution, by_label, execution_path.parent, classes)
    jarde_summary = validate_jarde(execution, by_label, classes, execution_path.parent)
    jadx_summary = validate_jadx(execution, by_label, classes, execution_path.parent)

    review = {
        'status': 'read_only_baseline_review_complete',
        'input': {'path': str(execution_path), 'bytes': len(execution_raw), 'sha256': sha(execution_raw)},
        'checker': {'path': str(Path(__file__).resolve()), 'bytes': Path(__file__).stat().st_size,
                    'sha256': sha(Path(__file__).read_bytes())},
        'capture': {'collector_status': execution['status'], 'source_unchanged': execution['source_unchanged'],
                    'target_absent': execution['target_absent'], 'finished_utc': execution['finished_utc']},
        'raw_commands': raw_summary,
        'source_dependency_tool_pins': source_checks,
        'compiled_profiles': class_summary,
        'runtime_oracles': runtime_summary,
        'jarde_api': jarde_summary,
        'jadx_extractions': jadx_summary,
        'limitations': [
            'This is an independent review of baseline capture evidence, not an implementation acceptance or before/after product comparison.',
            'Jarde does not expose private SSA in the public class-source report; this review validates class/method/BCI source-map facts only and makes no SSA claim.',
            'Fallback methods are allowed to map only emitted explanation origins. Missing physical BCIs are listed; only structured methods are required to map every Code BCI.',
            'JADX extraction and Jarde API exit/outcome are reported as observed. The checker does not require them to succeed or require testFor()I to be structured.',
            'The earlier prep source-pins.json belongs to an older Luna draft and is deliberately not used as authority; root-v3 execution.json and the copied collector.py are checked directly.',
        ],
    }
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with output_path.open('x', encoding='utf-8') as stream:
        json.dump(review, stream, ensure_ascii=False, indent=2, sort_keys=True)
        stream.write('\n')
    print(json.dumps({'status': review['status'], 'commands': raw_summary['command_count'],
                      'raw_streams': raw_summary['raw_stream_count'], 'Jarde_cells': jarde_summary['cell_count'],
                      'JADX_cells': jadx_summary['cell_count'], 'review': str(output_path)}, ensure_ascii=False))
    return 0


if __name__ == '__main__':
    try:
        raise SystemExit(main())
    except (OSError, ValueError, KeyError, TypeError, zipfile.BadZipFile, json.JSONDecodeError) as error:
        raise SystemExit(f'TestNestedLoops5 baseline review failed: {error}')
