#!/usr/bin/env python3
"""Read-only verifier for the prepared finally full-class comparison evidence."""
from __future__ import annotations

import hashlib
import json
import re
import sys
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
CHANGE_RESULTS = ROOT / 'openspec/changes/recover-proved-finally-cleanup/results'
REPLAY = CHANGE_RESULTS / 'finally-final-gates-root-v2'
INSPECTION = CHANGE_RESULTS / 'root-finally-member-inspection-v1'
INSPECTION_RESULT = INSPECTION / 'result.json'
RETURNED_RESULTS = ROOT / 'openspec/changes/recover-returned-int-array-compound-updates/results'
METADATA_PATH = RETURNED_RESULTS / 'candidate-cli-v2.json'
JDK_MANIFEST = ROOT / 'openspec/changes/recover-nested-int-array-compound-updates/results/returned-next-baseline-v1/manifest.json'
OUTPUT = CHANGE_RESULTS / 'finally-final-root-verification-v3.json'

PRODUCTS = {
    'crates/jarde-java/src/init.rs', 'crates/jarde-java/src/report.rs',
    'crates/jarde-java/src/build.rs', 'crates/jarde-java/src/ast.rs',
    'crates/jarde-java/src/emit.rs', 'crates/jarde-java/src/asserts.rs',
    'crates/jarde-java/src/field.rs', 'src/facade.rs', 'src/class_source.rs', 'Cargo.lock',
}
TESTS = {
    'tests/p3_nested_int_array_compound_updates.rs', '.github/workflows/ci.yml',
    'tests/recover_lambda_primitive_array_capture.rs',
    'tests/p3_constructed_reference_array_elements.rs', 'tests/recover_boxed_number_widening.rs',
}
INPUTS = {
    'FinallyNormal': {
        'source': ('openspec/evidence/java-syntax-2026-09-22/finally-completion/non-overriding/FinallyNormal.java',
                   'e11b663eff5513e79d75f4b6c6c1939b709e20ea9f8a39b8641edff087eb27e3'),
        'runner': ('openspec/evidence/java-syntax-2026-09-22/finally-completion/non-overriding/FinallyNormalRunner.java',
                   '414fc96c8bfa6698db24ad1141611d055a560f5598053a6bab697a27e9f64e80'),
        'jadx': ('openspec/evidence/java-syntax-2026-09-22/finally-completion/non-overriding/jadx.java.txt',
                 'ce6dcebb2af915c79f5a36b400cc48fabac0029244920112cd1f1099bc347f51'),
    },
    'FinallyCompletion': {
        'source': ('openspec/evidence/java-syntax-2026-09-22/finally-completion/FinallyCompletion.java',
                   '543d8bd6252aae4dd072f1d5a4289bd72282b0f94e5782a7e6b066ab0e33404e'),
        'runner': ('openspec/evidence/java-syntax-2026-09-22/finally-completion/FinallyCompletionRunner.java',
                   'b4d12e2b8d80261985baaee0df783cd31a3991855b195d862ae1569c9cec1db5'),
        'jadx': ('openspec/evidence/java-syntax-2026-09-22/finally-completion/jadx.java.txt',
                 '495218a394e5a6652dafc8e54d74dd2e3c5fd58713c77904b89acf7966f5844b'),
    },
}
JDK_LEGS = ('javac8', 'javac23')
KINDS = ('source', 'jadx', 'jarde')
NORMAL_OUTPUT = (
    b'normal:return:1:12\n'
    b'throw:java.lang.IllegalArgumentException:same=true:12\n'
)
COMPLETION_SOURCE_MARKERS = (b'throw-overrides:throw:java.lang.IllegalStateException:try=false:finally=true:56\n',
                            b'try-throw-preserved:throw:java.lang.IllegalArgumentException:try=true:finally=false:78\n')
COMPLETION_JADX_MARKERS = (b'throw-overrides:throw:java.lang.IllegalStateException:try=false:finally=true:566\n',
                           b'try-throw-preserved:throw:java.lang.IllegalArgumentException:try=true:finally=false:8\n')
ACCESS_BITS = {
    'ACC_PUBLIC': 0x0001, 'ACC_PRIVATE': 0x0002, 'ACC_PROTECTED': 0x0004,
    'ACC_STATIC': 0x0008, 'ACC_FINAL': 0x0010, 'ACC_SYNCHRONIZED': 0x0020,
    'ACC_VOLATILE': 0x0040, 'ACC_BRIDGE': 0x0040, 'ACC_TRANSIENT': 0x0080,
    'ACC_VARARGS': 0x0080, 'ACC_NATIVE': 0x0100, 'ACC_INTERFACE': 0x0200,
    'ACC_ABSTRACT': 0x0400, 'ACC_STRICT': 0x0800, 'ACC_SYNTHETIC': 0x1000,
    'ACC_ANNOTATION': 0x2000, 'ACC_ENUM': 0x4000, 'ACC_MODULE': 0x8000,
}


class VerifyFailure(Exception):
    pass


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def expect(condition: bool, label: str, errors: list[str], checks: list[int]) -> None:
    checks[0] += 1
    if not condition:
        errors.append(label)


def resolve_record(base: Path, row: dict) -> Path:
    path = Path(row['path'])
    return path if path.is_absolute() else base / path


def read_record(base: Path, row: dict) -> bytes:
    return resolve_record(base, row).read_bytes()


def record_ok(base: Path, row: dict) -> bool:
    try:
        data = read_record(base, row)
        return len(data) == row['bytes'] and sha(data) == row['sha256']
    except (KeyError, OSError, TypeError):
        return False


def jdk_home(tool: dict) -> str:
    return str(Path(tool['path']).parent.parent)


def command_stream_equal(left: dict, right: dict, base: Path) -> bool:
    try:
        return (left['exit'] == right['exit'] and
                read_record(base, left['stdout']) == read_record(base, right['stdout']) and
                read_record(base, left['stderr']) == read_record(base, right['stderr']))
    except (KeyError, OSError, TypeError):
        return False


def cli_member_rows(document: dict, category: str) -> list[tuple[str, str, int]]:
    rows = []
    for member in document.get(category, []):
        item = member.get('item', {})
        name = item.get('name', {}).get('escaped')
        descriptor = item.get('descriptor', {}).get('escaped')
        flags = item.get('access_flags')
        if not isinstance(name, str) or not isinstance(descriptor, str) or not isinstance(flags, int):
            raise ValueError(f'incomplete {category} member identity')
        rows.append((name, descriptor, flags))
    return rows


def parse_javap_members(text: str, class_name: str) -> dict:
    lines = text.splitlines()
    headers = []
    for index, line in enumerate(lines):
        stripped = line.strip()
        if not line.startswith('  ') or line.startswith('   ') or not stripped or stripped.startswith('#') or not stripped.endswith(';'):
            continue
        if stripped == 'static {};':
            headers.append((index, 'method', '<clinit>'))
        elif re.search(r'\([^)]*\)(?:\s+throws\s+[^;]+)?;$', stripped):
            prefix = stripped[:stripped.index('(')].split()
            if prefix:
                name = prefix[-1]
                if name == class_name:
                    name = '<init>'
                headers.append((index, 'method', name))
        elif '(' not in stripped:
            names = stripped[:-1].split()
            if len(names) >= 2:
                headers.append((index, 'field', names[-1]))

    members = {'fields': [], 'methods': []}
    for slot, (start, kind, name) in enumerate(headers):
        end = headers[slot + 1][0] if slot + 1 < len(headers) else len(lines)
        block = lines[start:end]
        descriptor_match = next((re.search(r'^\s*descriptor:\s*(\S+)\s*$', line) for line in block
                                if re.search(r'^\s*descriptor:\s*\S+\s*$', line)), None)
        flags_line = next((line for line in block if re.search(r'^\s*flags:\s*', line)), None)
        if descriptor_match is None or flags_line is None:
            raise ValueError(f'javap member lacks descriptor/flags: {name}')
        descriptor = descriptor_match.group(1)
        names = re.findall(r'ACC_[A-Z_]+', flags_line)
        flags = 0
        for flag in names:
            if flag not in ACCESS_BITS:
                raise ValueError(f'unknown javap access flag {flag}')
            flags |= ACCESS_BITS[flag]
        members[kind + 's'].append({'name': name, 'descriptor': descriptor, 'flags': flags,
                                    'has_code': any(line.strip() == 'Code:' for line in block),
                                    'block': block})
    return members


def source_map_bcis(method: dict) -> set[int]:
    mapped = set()
    report = method.get('outcome', {}).get('report', {})
    for segment in (report.get('source_map') or {}).get('segments', []):
        origin = segment.get('origin', {})
        primary = origin.get('primary', {})
        if isinstance(primary.get('bci'), int):
            mapped.add(primary['bci'])
        for derived in origin.get('derived', []):
            if isinstance(derived.get('bci'), int):
                mapped.add(derived['bci'])
    return mapped


def actual_bcis(method: dict) -> set[int]:
    result = set()
    code_seen = False
    for line in method['block']:
        if line.strip() == 'Code:':
            code_seen = True
            continue
        if code_seen:
            match = re.match(r'^\s*(\d+):\s+[a-z][a-z0-9_]*\b', line)
            if match:
                result.add(int(match.group(1)))
            elif line.strip() in ('Exception table:',) or line.strip().startswith(('StackMapTable:', 'LineNumberTable:')):
                break
    return result


def main() -> int:
    errors: list[str] = []
    checks = [0]
    manifest_path = REPLAY / 'manifest.json'
    manifest_raw = manifest_path.read_bytes()
    manifest = json.loads(manifest_raw)
    expect(manifest.get('schema') == 'finally-final-gates-current-cli-replay-root-v2', 'replay manifest schema', errors, checks)
    expect(manifest.get('status') == 'evidence_recorded' and manifest.get('output_root') == str(REPLAY), 'replay manifest status/path', errors, checks)
    expect(manifest.get('actual_case_count') == manifest.get('expected_case_count') == 12 and len(manifest.get('cases', [])) == 12,
           'exact twelve replay legs', errors, checks)
    expect(manifest.get('actual_cli_render_count') == manifest.get('expected_cli_render_count') == 4,
           'exact four CLI render calls', errors, checks)
    expect(not manifest.get('unexpected_cli_failures'), 'all four CLI calls returned zero', errors, checks)
    expect(manifest.get('environment_removed') == ['JAVA_TOOL_OPTIONS', '_JAVA_OPTIONS', 'JDK_JAVA_OPTIONS', 'CLASSPATH'],
           'clean JVM/classpath environment', errors, checks)

    actual_files = {path.relative_to(REPLAY).as_posix() for path in REPLAY.rglob('*')
                    if path.is_file() and path.name != 'manifest.json'}
    file_rows = {row['path']: row for row in manifest.get('files', [])}
    expect(actual_files == set(file_rows), 'replay artifact inventory is closed', errors, checks)
    for relative, row in file_rows.items():
        expect(record_ok(REPLAY, row), f'replay artifact size/hash: {relative}', errors, checks)

    metadata_path = Path(manifest['candidate_cli']['metadata_path'])
    metadata_raw = metadata_path.read_bytes()
    metadata = json.loads(metadata_raw)
    expect(sha(metadata_raw) == 'ddfbe127890d95ced5c4ed2a89d67d42deb7eba19e6c1e4ce12707221bd8f769', 'pinned metadata digest', errors, checks)
    expect(metadata['cli_sha256'] == '71f0a864243e7c4155789e05a0c5021ec06e87ac8a8bf9dfb38b38acb4906110', 'pinned CLI digest', errors, checks)
    cli = Path(manifest['candidate_cli']['path'])
    expect(manifest['candidate_cli']['metadata_sha256'] == sha(metadata_raw), 'candidate metadata hash', errors, checks)
    expect(manifest['candidate_cli']['sha256'] == metadata['cli_sha256'] and cli == Path(metadata['cli_path']) and
           cli.is_file() and sha(cli.read_bytes()) == metadata['cli_sha256'], 'frozen CLI path and binary identity', errors, checks)
    expect(set(metadata['candidate_sources']) == PRODUCTS and set(metadata['test_sources']) == TESTS,
           'candidate metadata exact 10 product/5 test source sets', errors, checks)
    expect(manifest['candidate_cli']['candidate_sources'] == metadata['candidate_sources'] and
           manifest['candidate_cli']['test_sources'] == metadata['test_sources'], 'replay binds source metadata', errors, checks)
    for category in ('candidate_sources', 'test_sources'):
        for relative, expected in metadata[category].items():
            try:
                current_hash = sha((ROOT / relative).read_bytes())
            except OSError:
                current_hash = None
            expect(current_hash == expected, f'current CLI/product source identity: {relative}', errors, checks)

    historical = manifest['historical_jdk_manifest']
    baseline_raw = JDK_MANIFEST.read_bytes()
    baseline = json.loads(baseline_raw)
    expect(sha(baseline_raw) == '10a6ea368ba5304594909d0318d650b7ad7c580904d0897c2cf7ba13232db4a3', 'pinned JDK manifest digest', errors, checks)
    expect(historical.get('path') == str(JDK_MANIFEST) and historical.get('sha256') == sha(baseline_raw) and
           historical.get('schema') == baseline.get('schema') == 'returned-array-update-next-baseline-v1',
           'fixed historical JDK manifest identity', errors, checks)
    for leg, index in (('javac8', 0), ('javac23', 1)):
        tools = baseline['jdk_tools'][index]
        expect(historical.get('jdk_tools', {}).get(leg) == tools, f'{leg} frozen JDK tool records', errors, checks)
        for name, row in tools.items():
            try:
                binary = Path(row['path']).read_bytes()
                good = len(binary) == row['bytes'] and sha(binary) == row['sha256']
            except OSError:
                good = False
            expect(good, f'{leg} frozen {name} binary', errors, checks)

    inputs_manifest = manifest.get('frozen_inputs', {})
    for family, entries in INPUTS.items():
        expect(family in inputs_manifest, f'{family} frozen input family', errors, checks)
        for kind, (relative, digest) in entries.items():
            row = inputs_manifest.get(family, {}).get(kind, {})
            try:
                frozen = (ROOT / relative).read_bytes()
                copied = read_record(REPLAY, row.get('copy', {}))
                good = sha(frozen) == digest == row.get('sha256') and copied == frozen and record_ok(REPLAY, row.get('copy', {}))
            except (OSError, KeyError, TypeError):
                good = False
            expect(good, f'{family} {kind} fixed path/hash/copied bytes', errors, checks)

    cases = {case['label']: case for case in manifest.get('cases', [])}
    expected_labels = {f'{family}-{leg}-{kind}' for family in INPUTS for leg in JDK_LEGS for kind in KINDS}
    expect(set(cases) == expected_labels, 'exact family/JDK/kind case closure', errors, checks)
    commands = manifest.get('commands', [])
    by_command = {command['label']: command for command in commands}
    expected_command_labels = set()
    for family in INPUTS:
        for leg in JDK_LEGS:
            for kind in ('source', 'jadx'):
                expected_command_labels.update((f'{family}-{leg}-{kind}-compile', f'{family}-{leg}-{kind}-run'))
            expected_command_labels.update((f'{family}-{leg}-jarde-render', f'{family}-{leg}-jarde-compile'))
            if family == 'FinallyNormal':
                expected_command_labels.add(f'{family}-{leg}-jarde-run')
    expect(len(commands) == 26 and set(by_command) == expected_command_labels, 'exact 26 actual subprocess records', errors, checks)
    for command in commands:
        for stream in ('stdout', 'stderr'):
            expect(record_ok(REPLAY, command.get(stream, {})),
                   f"actual {command.get('label')} {stream} raw bytes/hash", errors, checks)
        expect(command.get('environment_removed') == manifest.get('environment_removed'),
               f"actual {command.get('label')} clean environment", errors, checks)

    by_oracle = manifest.get('source_oracles', {})
    for label, case in cases.items():
        family, leg, kind = case['family'], case['jdk_leg'], case['kind']
        expect(label == f'{family}-{leg}-{kind}', f'{label} label metadata', errors, checks)
        tools = baseline['jdk_tools'][0 if leg == 'javac8' else 1]
        home = jdk_home(tools['java'])
        expect(case.get('fresh_jdk_tools') == tools and case.get('java_home') == home,
               f'{label} fixed JDK identity', errors, checks)
        classes_dir = Path(case['classes_dir'])
        empty = Path(case['empty_classpath_sourcepath'])
        expect(empty.is_dir() and not any(empty.iterdir()), f'{label} actual empty CP/SP directories', errors, checks)
        compile = case.get('compile')
        compile_cmd = by_command.get(f'{label}-compile')
        if case.get('available'):
            expect(compile is not None and compile == compile_cmd, f'{label} compile row binds actual command', errors, checks)
            argv = compile['argv']
            expect(argv[0] == tools['javac']['path'] and compile.get('java_home') == home,
                   f'{label} actual frozen javac path/home', errors, checks)
            expected_flags = ['-source', '8', '-target', '8', '-g:none', '-Xlint:-options']
            try:
                cp_index = argv.index('-classpath')
                sp_index = argv.index('-sourcepath')
                d_index = argv.index('-d')
                argv_flags = argv[1:cp_index]
                argv_empty = argv[cp_index + 1] == str(empty) and argv[sp_index + 1] == str(empty)
                argv_classes = Path(argv[d_index + 1]) == classes_dir
                source_args = [Path(value) for value in argv[d_index + 2:]]
            except (ValueError, IndexError):
                argv_flags, argv_empty, argv_classes, source_args = [], False, False, []
            expected_sources = [REPLAY / row['path'] for row in case.get('compile_sources', [])]
            expect(argv_flags == expected_flags and case.get('compiler_flags') == expected_flags,
                   f'{label} exact compiler flags', errors, checks)
            expect(argv_empty and argv_classes, f'{label} actual isolated empty CP/SP and private classes', errors, checks)
            expect(source_args == expected_sources and len(source_args) == 2,
                   f'{label} every complete source passed to javac', errors, checks)
            for source_row in case.get('compile_sources', []):
                expect(record_ok(REPLAY, source_row), f'{label} compilation source hash {source_row.get("path")}', errors, checks)
            runtime = case.get('runtime')
            if runtime is not None:
                runtime_cmd = by_command.get(f'{label}-run')
                expect(runtime == runtime_cmd, f'{label} runtime row binds actual command', errors, checks)
                runner_class = 'FinallyNormalRunner' if family == 'FinallyNormal' else 'FinallyCompletionRunner'
                main_class = 'defpackage.' + runner_class if kind == 'jadx' else runner_class
                expect(runtime['argv'] == [tools['java']['path'], '-Xverify:all', '-cp', str(classes_dir), main_class] and
                       runtime.get('java_home') == home, f'{label} runtime verifies only fresh classes', errors, checks)
            else:
                expect(f'{label}-run' not in by_command, f'{label} no runtime command after failed compilation', errors, checks)
            class_rows = case.get('candidate_classes', [])
            actual_class_paths = sorted(path.relative_to(classes_dir).as_posix() for path in classes_dir.rglob('*.class'))
            recorded_class_paths = sorted(row['path'] for row in class_rows)
            expect(actual_class_paths == recorded_class_paths, f'{label} generated class closure', errors, checks)
            for row in class_rows:
                expect(record_ok(classes_dir, row), f'{label} generated class hash {row.get("path")}', errors, checks)
            if runtime is not None:
                expect(len(class_rows) == 2, f'{label} only target and fixed Runner classes', errors, checks)
        else:
            expect(compile is None and case.get('runtime') is None, f'{label} unavailable without compilation', errors, checks)

        if kind == 'jarde':
            render = case.get('render')
            render_cmd = by_command.get(f'{label}-render')
            expect(render is not None and render == render_cmd and render.get('exit') == 0,
                   f'{label} actual CLI JSON command success', errors, checks)
            expected_argv = [manifest['candidate_cli']['path'], 'class-source', '--input',
                             str(REPLAY / 'cases' / label / 'input.class'), '--class', family,
                             '--policy', 'single-class', '--release', '8', '--format', 'json', '--evidence', 'all']
            expect(render.get('argv') == expected_argv, f'{label} exact CLI full-evidence argv', errors, checks)
            expect(case.get('report_json_parsed') is True and case.get('report_parse_error') is None,
                   f'{label} CLI JSON parses', errors, checks)
            document = json.loads(read_record(REPLAY, render['stdout']))
            expect(case.get('raw_json_stdout') == render['stdout'], f'{label} raw JSON reference', errors, checks)
            expect(document.get('declaration', {}).get('name') == family and document.get('class') == case.get('report_class') and document.get('text') is not None,
                   f'{label} full class-source document identity', errors, checks)
            source_row = case.get('report_text', {})
            expect(record_ok(REPLAY, source_row), f'{label} saved complete Java source hash', errors, checks)
            source_text = read_record(REPLAY, source_row)
            expect(source_text == document['text'].encode(), f'{label} saved source equals complete JSON text', errors, checks)
            expect(case.get('report_members') == document.get('methods'), f'{label} member JSON preserved completely', errors, checks)
            expect(len(document.get('methods', [])) == (4 if family == 'FinallyNormal' else 7),
                   f'{label} all physical methods preserved', errors, checks)
            expect(len(document.get('fields', [])) == 3, f'{label} all physical fields preserved', errors, checks)

    # Original source output is the live behavioral oracle; all process triples are raw bytes.
    for family in INPUTS:
        oracle_rows = [by_oracle[f'{family}-{leg}'] for leg in JDK_LEGS]
        expect(all(row.get('fresh_execution') is True and row.get('exit') == 0 for row in oracle_rows),
               f'{family} two fresh source oracles succeed', errors, checks)
        for oracle in oracle_rows:
            for stream in ('stdout', 'stderr'):
                expect(record_ok(REPLAY, oracle[stream]), f'{family} oracle {stream} raw/hash', errors, checks)
        expect(read_record(REPLAY, oracle_rows[0]['stdout']) == read_record(REPLAY, oracle_rows[1]['stdout']) and
               read_record(REPLAY, oracle_rows[0]['stderr']) == read_record(REPLAY, oracle_rows[1]['stderr']),
               f'{family} source oracle raw output agrees across JDKs', errors, checks)

    for leg in JDK_LEGS:
        source = cases[f'FinallyNormal-{leg}-source']
        jarde = cases[f'FinallyNormal-{leg}-jarde']
        jadx = cases[f'FinallyNormal-{leg}-jadx']
        expect(source['runtime']['exit'] == 0 and
               read_record(REPLAY, source['runtime']['stdout']) == NORMAL_OUTPUT and
               read_record(REPLAY, source['runtime']['stderr']) == b'',
               f'FinallyNormal {leg} original oracle bytes', errors, checks)
        for compare, label in ((jarde, 'Jarde'), (jadx, 'historical JADX')):
            expect(compare['compile']['exit'] == source['compile']['exit'] and
                   command_stream_equal(compare['compile'], source['compile'], REPLAY),
                   f'FinallyNormal {leg} {label} compile triple matches source', errors, checks)
            expect(compare['runtime'] is not None and command_stream_equal(compare['runtime'], source['runtime'], REPLAY),
                   f'FinallyNormal {leg} {label} runtime exit/stdout/stderr matches source', errors, checks)

        completion_source = cases[f'FinallyCompletion-{leg}-source']
        completion_jadx = cases[f'FinallyCompletion-{leg}-jadx']
        completion_jarde = cases[f'FinallyCompletion-{leg}-jarde']
        source_out = read_record(REPLAY, completion_source['runtime']['stdout'])
        jadx_out = read_record(REPLAY, completion_jadx['runtime']['stdout'])
        expect(source_out == b'normal:return:1:12\nreturn-overrides:return:4:34\nthrow-overrides:throw:java.lang.IllegalStateException:try=false:finally=true:56\ntry-throw-preserved:throw:java.lang.IllegalArgumentException:try=true:finally=false:78\n' and completion_source['runtime']['exit'] == 0 and read_record(REPLAY, completion_source['runtime']['stderr']) == b'',
               f'FinallyCompletion {leg} original return/throw completion oracle', errors, checks)
        expect(all(marker in jadx_out for marker in COMPLETION_JADX_MARKERS) and jadx_out != source_out,
               f'FinallyCompletion {leg} historical JADX records 566/8 misbehavior', errors, checks)
        expect(completion_jadx['matches_source_oracle_exit_stdout_stderr'] is False,
               f'FinallyCompletion {leg} JADX is not accepted as oracle', errors, checks)
        expect(completion_jarde['compile'] is not None and completion_jarde['compile']['exit'] != 0 and
               completion_jarde['runtime'] is None and any(needle in read_record(REPLAY, completion_jarde['compile']['stderr']) for needle in (b'missing return statement', '缺少返回语句'.encode())),
               f'FinallyCompletion {leg} complete-class missing-return compile failure remains a negative case', errors, checks)

    # Inspect the four actual javap -p -s -v outputs and compare physical signatures to CLI JSON.
    inspection_rows = json.loads(INSPECTION_RESULT.read_bytes())
    expected_inspection_labels = {f'{family}-{leg}-source' for family in INPUTS for leg in JDK_LEGS}
    inspection_by_label = {row['label']: row for row in inspection_rows}
    expect(set(inspection_by_label) == expected_inspection_labels and len(inspection_rows) == 4,
           'exact four physical javap inspections', errors, checks)
    actual_inspection_files = {path.name for path in INSPECTION.iterdir() if path.is_file() and path.name != 'result.json'}
    expected_inspection_files = {row[stream]['path'] for row in inspection_rows for stream in ('stdout', 'stderr')}
    expect(actual_inspection_files == expected_inspection_files, 'javap inspection raw file closure', errors, checks)
    for label, inspection in inspection_by_label.items():
        family = 'FinallyNormal' if label.startswith('FinallyNormal-') else 'FinallyCompletion'
        leg = 'javac8' if '-javac8-' in label else 'javac23'
        source_case = cases[label]
        candidate_case = cases[f'{family}-{leg}-jarde']
        tools = baseline['jdk_tools'][0 if leg == 'javac8' else 1]
        javap_path = Path(inspection['argv'][0])
        expect(inspection.get('exit') == 0 and str(javap_path) == tools['javap']['path'] and
               inspection['argv'][1:4] == ['-p', '-s', '-v'], f'{label} actual frozen javap invocation', errors, checks)
        class_path = REPLAY / 'cases' / label / 'classes' / f'{family}.class'
        try:
            class_bytes = class_path.read_bytes()
        except OSError:
            class_bytes = b''
        expect(inspection['argv'][-1] == str(class_path) and len(class_bytes) == inspection['class_bytes'] and
               sha(class_bytes) == inspection['class_sha256'], f'{label} javap class bytes identity', errors, checks)
        for stream in ('stdout', 'stderr'):
            try:
                stream_bytes = read_record(INSPECTION, inspection[stream])
                valid = len(stream_bytes) == inspection[stream]['bytes'] and sha(stream_bytes) == inspection[stream]['sha256']
            except (KeyError, OSError, TypeError):
                stream_bytes, valid = b'', False
            expect(valid, f'{label} javap raw {stream} hash', errors, checks)
        javap_text = read_record(INSPECTION, inspection['stdout']).decode()
        physical = parse_javap_members(javap_text, family)
        doc = json.loads(read_record(REPLAY, candidate_case['render']['stdout']))
        cli_fields = cli_member_rows(doc, 'fields')
        cli_methods = cli_member_rows(doc, 'methods')
        physical_fields = [(row['name'], row['descriptor'], row['flags']) for row in physical['fields']]
        physical_methods = [(row['name'], row['descriptor'], row['flags']) for row in physical['methods']]
        expect(Counter(cli_fields) == Counter(physical_fields), f'{label} CLI field names/descriptors/flags equal javap -p', errors, checks)
        expect(Counter(cli_methods) == Counter(physical_methods), f'{label} CLI method names/descriptors/flags equal javap -p', errors, checks)
        required_method_count = 4 if family == 'FinallyNormal' else 7
        expect(len(physical['methods']) == required_method_count and len(physical['fields']) == 3,
               f'{label} physical member counts (including private mark)', errors, checks)
        mark = next((row for row in physical['methods'] if row['name'] == 'mark'), None)
        expect(mark is not None and mark['descriptor'] == '(I)I' and mark['flags'] == (0x0002 | 0x0008) and mark['has_code'],
               f'{label} private static mark method and Code attribute', errors, checks)
        expect(sum(row['has_code'] for row in physical['methods']) == required_method_count,
               f'{label} actual Code-bearing methods are {required_method_count}', errors, checks)
        for method in doc['methods']:
            expect(method.get('item', {}).get('body', {}).get('kind') == 'code_attribute',
                   f'{label} CLI keeps Code-bodied method {method.get("item", {}).get("name", {}).get("escaped")}', errors, checks)

        if family == 'FinallyNormal':
            run_physical = next(row for row in physical['methods'] if row['name'] == 'run' and row['descriptor'] == '()I')
            run_cli = next(row for row in doc['methods'] if row['item']['name']['escaped'] == 'run' and
                           row['item']['descriptor']['escaped'] == '()I')
            real = actual_bcis(run_physical)
            mapped = source_map_bcis(run_cli)
            expect(bool(real) and real == mapped, f'{label} run() source-map BCI set equals all physical instruction BCIs', errors, checks)

    report = {
        'schema': 'finally-final-root-verification-v3',
        'status': 'passed' if not errors else 'failed',
        'checks': checks[0],
        'errors': errors,
        'replay_manifest_sha256': sha((REPLAY / 'manifest.json').read_bytes()),
        'javap_inspection_sha256': sha(INSPECTION_RESULT.read_bytes()),
        'case_count': len(cases), 'command_count': len(commands), 'javap_count': len(inspection_rows),
        'physical_code_methods': {'FinallyNormal': 4, 'FinallyCompletion': 7},
        'normal_jarde_and_jadx_raw_matches': True if not errors else None,
        'completion_jadx_negative_566_8': True if not errors else None,
        'completion_jarde_compile_negative': True if not errors else None,
    }
    if OUTPUT.exists():
        raise SystemExit(f'refusing to overwrite {OUTPUT}')
    OUTPUT.write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps(report, ensure_ascii=False, indent=2))
    return 0 if not errors else 1


if __name__ == '__main__':
    raise SystemExit(main())
