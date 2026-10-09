#!/usr/bin/env python3
"""Independent read-only verifier for returned candidate replay evidence."""
from __future__ import annotations

import hashlib
import importlib.util
import json
import sys
from pathlib import Path

ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
HERE = Path(__file__).resolve().parent
BASELINE = ROOT / 'openspec/changes/recover-nested-int-array-compound-updates/results/returned-next-baseline-v1'
BASELINE_MANIFEST = BASELINE / 'manifest.json'
FIXTURE = ROOT / 'tests/fixtures/returned-int-array-compound-updates'
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
METHODS = {'<init>', 'plain2', 'plain3', 'scalar', 'traced', 'row', 'index', 'rhs', 'swap', 'replaceRow', 'different'}
COMPOUND = ('plain2', 'plain3', 'scalar', 'traced', 'replaceRow')
SCALAR_BCI_OPCODES = {2: 0x5c, 3: 0x2e, 5: 0x60, 6: 0x5b, 7: 0x4f, 8: 0xac}


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def expect(condition: bool, message: str, errors: list[str], checks: list[int]) -> None:
    checks[0] += 1
    if not condition:
        errors.append(message)


def resolve(path: str, base: Path) -> Path:
    value = Path(path)
    return value if value.is_absolute() else base / value


def raw(path: str, base: Path) -> bytes:
    return resolve(path, base).read_bytes()


def verify_record(base: Path, row: dict) -> bool:
    try:
        data = raw(row['path'], base)
        return len(data) == row['bytes'] and sha(data) == row['sha256']
    except (KeyError, OSError, TypeError):
        return False


def load_parser():
    path = ROOT / 'openspec/changes/recover-covariant-child-array-initializers/results/prepare-child-jvm-controls-v1.py'
    sys.dont_write_bytecode = True
    spec = importlib.util.spec_from_file_location('returned_candidate_verifier_parser', path)
    if spec is None or spec.loader is None:
        raise SystemExit(f'cannot load read-only physical class parser: {path}')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def source_map_bcis(row: dict) -> set[int]:
    result: set[int] = set()
    for segment in (row.get('source_map') or {}).get('segments', []):
        origin = segment.get('origin', {})
        primary = origin.get('primary', {})
        if isinstance(primary.get('bci'), int):
            result.add(primary['bci'])
        for derived in origin.get('derived', []):
            if isinstance(derived.get('bci'), int):
                result.add(derived['bci'])
    return result


def records_inside(value):
    if isinstance(value, dict):
        if {'path', 'bytes', 'sha256'} <= set(value):
            yield value
        else:
            for child in value.values():
                yield from records_inside(child)
    elif isinstance(value, list):
        for child in value:
            yield from records_inside(child)


def main() -> int:
    if len(sys.argv) != 4:
        raise SystemExit('usage: verify-returned-candidate-luna-v1.py OUTPUT_DIR FROZEN_CLI CLI_METADATA_JSON')
    out, cli, metadata_path = (Path(arg).resolve() for arg in sys.argv[1:])
    manifest_path = out / 'manifest.json'
    errors: list[str] = []
    checks = [0]
    if not manifest_path.is_file():
        raise SystemExit(f'missing replay manifest: {manifest_path}')
    manifest_raw = manifest_path.read_bytes()
    manifest = json.loads(manifest_raw)
    baseline_raw = BASELINE_MANIFEST.read_bytes()
    baseline = json.loads(baseline_raw)
    metadata_raw = metadata_path.read_bytes()
    metadata = json.loads(metadata_raw)
    parser = load_parser()

    expect(manifest.get('schema') == 'returned-int-array-compound-candidate-luna-v1', 'manifest schema', errors, checks)
    expect(manifest.get('status') == 'complete_replay_recorded' and manifest.get('output_root') == str(out), 'replay status/output root', errors, checks)
    expect(manifest.get('expected_denominator') == 2 and len(manifest.get('cases', [])) == 2, 'exact two-JDK denominator', errors, checks)
    expect(manifest.get('environment_removed') == ['JAVA_TOOL_OPTIONS', '_JAVA_OPTIONS', 'JDK_JAVA_OPTIONS', 'CLASSPATH'], 'ambient JVM/classpath variables removed', errors, checks)
    expect(manifest.get('runner', {}).get('sha256') == sha((HERE / 'replay-returned-candidate-luna-v1.py').read_bytes()), 'replay runner source identity', errors, checks)

    cli_row = manifest.get('candidate_cli', {})
    expect(cli_row.get('path') == str(cli) and cli.is_file() and sha(cli.read_bytes()) == metadata.get('cli_sha256') == cli_row.get('sha256'), 'candidate CLI bytes/path identity', errors, checks)
    expect(cli_row.get('metadata_path') == str(metadata_path) and sha(metadata_raw) == cli_row.get('metadata_sha256'), 'CLI metadata path/hash identity', errors, checks)
    expect(Path(metadata.get('cli_path', '')).resolve() == cli, 'metadata canonical CLI path', errors, checks)
    expect(set(metadata.get('candidate_sources', {})) == PRODUCTS and set(metadata.get('test_sources', {})) == TESTS, 'exact frozen source metadata closure', errors, checks)
    expect(cli_row.get('candidate_sources') == metadata.get('candidate_sources') and cli_row.get('test_sources') == metadata.get('test_sources'), 'manifest source metadata equals CLI metadata', errors, checks)
    for relative, digest in {**metadata.get('candidate_sources', {}), **metadata.get('test_sources', {})}.items():
        try:
            actual = sha((ROOT / relative).read_bytes())
        except OSError:
            actual = None
        expect(actual == digest, f'current candidate/test source identity {relative}', errors, checks)
    canonical = metadata.get('canonical_files', {})
    expect(bool(canonical) and cli_row.get('canonical_files') == canonical, 'canonical fixture inventory bound', errors, checks)
    for relative, digest in canonical.items():
        try:
            actual = sha((ROOT / relative).read_bytes())
        except OSError:
            actual = None
        expect(actual == digest, f'canonical fixture identity {relative}', errors, checks)

    expect(baseline.get('schema') == 'returned-array-update-next-baseline-v1' and
           baseline.get('scope') == 'next-candidate analysis only; no product/spec implementation', 'historical baseline scope/schema', errors, checks)
    historical_ref = manifest.get('historical_baseline', {})
    expect(historical_ref.get('path') == str(BASELINE_MANIFEST) and historical_ref.get('sha256') == sha(baseline_raw) and
           historical_ref.get('fresh_execution_claim') is False, 'historical baseline manifest is hash-bound and reused', errors, checks)
    inventory = {row['path']: row for row in baseline.get('files', [])}
    for relative, row in inventory.items():
        expect(verify_record(BASELINE, row), f'historical baseline inventory {relative}', errors, checks)
    # The replay binds the prior baseline manifest and every prior output via the baseline inventory.
    expect(len(manifest.get('historical_cases', [])) == 6, 'six prior original/JADX records reused', errors, checks)
    baseline_cases = {row['label']: row for row in baseline.get('cases', [])}
    history = {row['label']: row for row in manifest.get('historical_cases', [])}
    expected_history = {label for label, row in baseline_cases.items() if row.get('kind') in ('original', 'jadx')}
    expect(set(history) == expected_history, 'exact historical original/JADX case labels', errors, checks)
    for label in sorted(expected_history):
        historic = baseline_cases[label]
        stored = history[label]
        expect(stored.get('fresh_execution') is False and stored.get('kind') == historic.get('kind'), f'{label} explicitly reused', errors, checks)
        expect(stored.get('sources') == historic.get('sources') and stored.get('classes') == historic.get('classes') and
               stored.get('compile') == historic.get('compile') and stored.get('runtime') == historic.get('runtime') and
               stored.get('decompile') == historic.get('decompile'), f'{label} historical original/JADX record equality', errors, checks)
        for row in records_inside(stored):
            expect(verify_record(BASELINE, row), f'{label} historical raw/source/class record {row.get("path")}', errors, checks)
    for leg in ('javac8', 'javac23'):
        toolrow = baseline['jdk_tools'][0 if leg == 'javac8' else 1]
        for name, row in toolrow.items():
            expect(verify_record(Path('/'), row), f'{leg} actual frozen {name} tool binary', errors, checks)

    fixture_inputs = manifest.get('fixture_inputs', {})
    wanted_fixture = {
        'source': FIXTURE / 'ReturnedIntArrayUpdates.java', 'runner': FIXTURE / 'Runner.java',
        'expected_stdout': FIXTURE / 'expected.stdout', 'expected_stderr': FIXTURE / 'expected.stderr',
    }
    for key, actual_path in wanted_fixture.items():
        row = fixture_inputs.get(key, {})
        expect(verify_record(out, row), f'copied fixture evidence {key} hash', errors, checks)
        try:
            expect(raw(row['path'], out) == actual_path.read_bytes(), f'exact original fixture bytes {key}', errors, checks)
        except (OSError, KeyError):
            expect(False, f'exact original fixture bytes {key}', errors, checks)
    oracle_out = (FIXTURE / 'expected.stdout').read_bytes()
    oracle_err = (FIXTURE / 'expected.stderr').read_bytes()
    expect(oracle_err == b'', 'fixture oracle stderr is empty', errors, checks)

    # Closed inventory covers every artifact and raw stream emitted by the replay.
    actual_files = {path.relative_to(out).as_posix() for path in out.rglob('*') if path.is_file() and path.name != 'manifest.json'}
    file_rows = {row['path']: row for row in manifest.get('files', [])}
    expect(actual_files == set(file_rows), 'replay output inventory is closed', errors, checks)
    for relative, row in file_rows.items():
        expect(verify_record(out, row), f'replay output hash {relative}', errors, checks)

    commands = manifest.get('commands', [])
    by_label = {command['label']: command for command in commands}
    cases = {case['label']: case for case in manifest.get('cases', [])}
    expect(set(cases) == {'javac8-jarde', 'javac23-jarde'}, 'exact candidate case labels', errors, checks)
    expect(len(commands) == 6 and set(by_label) == {f'{label}{suffix}' for label in cases for suffix in ('-render', '-compile', '-run')},
           'exact three actual subprocesses per JDK', errors, checks)
    for command in commands:
        expect(command.get('exit') == 0, f'actual command exit {command.get("label")}', errors, checks)
        expect(command.get('environment_removed') == manifest.get('environment_removed'), f'clean environment {command.get("label")}', errors, checks)
        for stream in ('stdout', 'stderr'):
            expect(verify_record(out, command.get(stream, {})), f'actual command raw {command.get("label")} {stream}', errors, checks)

    for label, case in cases.items():
        leg = case['leg']
        expect(leg in ('javac8', 'javac23') and case.get('candidate_success') is True and
               case.get('runtime_matches_original_exit_stdout_stderr') is True, f'{label} replay success summary', errors, checks)
        expect(case.get('original', {}).get('fresh_execution') is False and
               case.get('original_stream_source', {}).get('fresh_execution') is False, f'{label} original is historical, not fresh', errors, checks)
        expect(case.get('original_stream_source', {}).get('manifest') == str(BASELINE_MANIFEST), f'{label} historical original provenance', errors, checks)
        class_input = case.get('input', {})
        expect(verify_record(out, class_input), f'{label} copied class bytes/hash', errors, checks)
        historical_original = baseline_cases[f'{leg}-original']
        original_class = next(row for row in historical_original['classes'] if Path(row['path']).name == 'ReturnedIntArrayUpdates.class')
        try:
            expect(raw(class_input['path'], out) == raw(original_class['path'], BASELINE) and
                   class_input.get('sha256') == original_class['sha256'], f'{label} byte-exact historical original class', errors, checks)
        except (OSError, KeyError):
            expect(False, f'{label} byte-exact historical original class', errors, checks)
        expect(case.get('original_stream_source', {}).get('stdout') == historical_original['runtime']['stdout'] and
               case.get('original_stream_source', {}).get('stderr') == historical_original['runtime']['stderr'], f'{label} historical raw oracle identity', errors, checks)
        for stream, expected in (('stdout', oracle_out), ('stderr', oracle_err)):
            row = case['original'][stream]
            expect(verify_record(out, row) and raw(row['path'], out) == expected, f'{label} fixture/original {stream} bytes', errors, checks)
        expect(case['original'].get('exit') == historical_original['runtime']['exit'] == 0, f'{label} historical original exit', errors, checks)

        render = by_label[label + '-render']
        expected_render_argv = [str(cli), 'class-source', '--input', str(out / class_input['path']), '--class',
                               'ReturnedIntArrayUpdates', '--policy', 'single-class', '--release', '8', '--format', 'json', '--evidence', 'all']
        expect(render.get('argv') == expected_render_argv and render.get('java_home') is None, f'{label} exact class-source --evidence all argv', errors, checks)
        raw_document = json.loads(raw(render['stdout']['path'], out))
        source_row = case['report'].get('source', {})
        expect(case['report'].get('json_parsed') is True and case['report'].get('parse_error') is None, f'{label} renderer JSON parses', errors, checks)
        expect(verify_record(out, source_row), f'{label} complete generated Java source hash', errors, checks)
        source_text = raw(source_row['path'], out).decode()
        expect(raw_document.get('text') == source_text and sha(source_text.encode()) == case['report'].get('source_text_sha256'), f'{label} raw JSON binds full saved source', errors, checks)
        raw_method_rows = raw_document.get('methods', [])
        names = {row.get('item', {}).get('name', {}).get('escaped') for row in raw_method_rows}
        summary_by_name = {row.get('name'): row for row in case['report'].get('members', [])}
        expect(names == METHODS and set(summary_by_name) == METHODS and len(raw_method_rows) == 11, f'{label} all 11 physical methods retained', errors, checks)
        for raw_method in raw_method_rows:
            item = raw_method.get('item', {})
            name = item.get('name', {}).get('escaped')
            outcome = raw_method.get('outcome', {})
            report = outcome.get('report') if outcome.get('kind') == 'recovered' else None
            summary = summary_by_name.get(name, {})
            body = report.get('text', '') if report else ''
            expect(outcome.get('kind') == 'recovered' and report is not None, f'{label}/{name} recovered body', errors, checks)
            expect(not raw_method.get('markers') and 'bytecode' not in body.lower() and 'jarde_refused_body' not in body,
                   f'{label}/{name} no refusal/bytecode body', errors, checks)
            expect(summary.get('body') == body and summary.get('descriptor') == item.get('descriptor', {}).get('escaped') and
                   summary.get('source_map') == report.get('source_map') if report else False,
                   f'{label}/{name} raw JSON and manifest member equality', errors, checks)
        for name in COMPOUND:
            expect('return ' in summary_by_name.get(name, {}).get('body', '') and '+=' in summary_by_name.get(name, {}).get('body', ''),
                   f'{label}/{name} returned compound assignment body', errors, checks)

        # Decode the actual original class and prove source-map links reach physical opcodes.
        input_bytes = raw(class_input['path'], out)
        parsed = parser.parse_class(input_bytes)
        facts = next((row for row in manifest.get('physical_methods', []) if row.get('case') == label), {})
        physical_by_name = {row['name']: row for row in facts.get('methods', [])}
        for name in COMPOUND:
            descriptor = summary_by_name[name]['descriptor']
            code = parser.find_method_code(input_bytes, parsed, name, descriptor)
            instructions = parser.decode_instructions(input_bytes, code)
            physical_map = {ins['bci']: ins['opcode'] for ins in instructions}
            recorded = {row['bci']: row['opcode'] for row in physical_by_name.get(name, {}).get('instructions', [])}
            expect(recorded == physical_map, f'{label}/{name} recorded physical instruction inventory', errors, checks)
            anchors = {ins['bci'] for ins in instructions if ins['opcode'] in (0x5c, 0x2e, 0x60, 0x5b, 0x4f, 0x32, 0xb8)}
            expect(anchors <= source_map_bcis(summary_by_name[name]), f'{label}/{name} physical compound source-map anchors', errors, checks)
        scalar_instructions = {ins['bci']: ins['opcode'] for ins in parser.decode_instructions(
            input_bytes, parser.find_method_code(input_bytes, parsed, 'scalar', summary_by_name['scalar']['descriptor']))}
        expect(all(scalar_instructions.get(bci) == opcode for bci, opcode in SCALAR_BCI_OPCODES.items()),
               f'{label} scalar exact physical BCI dup2/iaload/iadd/dup_x2/iastore/ireturn', errors, checks)
        expect(set(SCALAR_BCI_OPCODES) <= source_map_bcis(summary_by_name['scalar']), f'{label} scalar exact BCI anchors mapped', errors, checks)

        compile = case.get('compile')
        compile_command = by_label[label + '-compile']
        tools = baseline['jdk_tools'][0 if leg == 'javac8' else 1]
        java_home = str(Path(tools['java']['path']).parent.parent)
        expected_flags = ['-source', '8', '-target', '8', '-g:none', '-Xlint:-options']
        empty = Path(case.get('empty_classpath_sourcepath', ''))
        compile_argv = compile.get('argv', []) if compile else []
        expected_sources = [out / row['path'] for row in case.get('compile_sources', [])]
        expect(compile == compile_command and compile is not None and compile.get('java_home') == java_home,
               f'{label} compile command record/JDK identity', errors, checks)
        expect(len(compile_argv) >= 1 and compile_argv[0] == tools['javac']['path'], f'{label} frozen javac path', errors, checks)
        expect(compile_argv[1:compile_argv.index('-classpath')] == expected_flags if '-classpath' in compile_argv else False,
               f'{label} compiler flags', errors, checks)
        expect(compile_argv[compile_argv.index('-classpath') + 1] == str(empty) and
               compile_argv[compile_argv.index('-sourcepath') + 1] == str(empty) and empty.is_dir() and not any(empty.iterdir()) if '-sourcepath' in compile_argv and '-classpath' in compile_argv else False,
               f'{label} explicit empty classpath/sourcepath', errors, checks)
        try:
            dindex = compile_argv.index('-d')
            classes_dir = Path(compile_argv[dindex + 1])
            source_args = [Path(value) for value in compile_argv[dindex + 2:]]
            expect(source_args == expected_sources and len(source_args) == 2, f'{label} compile every full source', errors, checks)
            expect(source_args[1].read_bytes() == (FIXTURE / 'Runner.java').read_bytes(), f'{label} exact canonical Runner source', errors, checks)
        except (ValueError, IndexError, OSError):
            classes_dir = Path('/definitely-missing-classes')
            expect(False, f'{label} compile source argv shape', errors, checks)
        expect(compile.get('exit') == 0, f'{label} compile exit zero', errors, checks)
        expect(all(verify_record(out, row) for row in case.get('compile_sources', [])), f'{label} compile source identities', errors, checks)

        runtime = case.get('runtime')
        runtime_command = by_label[label + '-run']
        expect(runtime == runtime_command and runtime is not None and runtime.get('java_home') == java_home,
               f'{label} runtime command record/JDK identity', errors, checks)
        runtime_argv = runtime.get('argv', []) if runtime else []
        expect(runtime_argv == [tools['java']['path'], '-Xverify:all', '-cp', str(classes_dir), 'Runner'],
               f'{label} verified JVM uses only fresh candidate classes', errors, checks)
        expect(runtime.get('exit') == case['original']['exit'], f'{label} runtime/original exit equality', errors, checks)
        for stream, expected in (('stdout', oracle_out), ('stderr', oracle_err)):
            expect(raw(runtime[stream]['path'], out) == expected, f'{label} actual runtime {stream} raw bytes', errors, checks)
        generated = sorted(classes_dir.rglob('*.class'))
        expect(sorted(path.name for path in generated) == ['ReturnedIntArrayUpdates.class', 'Runner.class'], f'{label} complete generated class closure', errors, checks)
        expect(len(case.get('candidate_classes', [])) == 2 and all(verify_record(classes_dir, row) for row in case['candidate_classes']),
               f'{label} generated class hashes', errors, checks)

    report = {'schema': 'returned-int-array-compound-candidate-luna-verification-v1',
              'status': 'passed' if not errors else 'failed', 'checks': checks[0], 'errors': errors,
              'manifest_sha256': sha(manifest_raw), 'verifier_sha256': sha(Path(__file__).read_bytes()),
              'candidate_cli_sha256': cli_row.get('sha256'), 'case_count': len(cases),
              'successful_case_count': sum(case.get('candidate_success') is True for case in cases.values()),
              'historical_jadx_cases_reused': sum(row.get('kind') == 'jadx' and row.get('fresh_execution') is False for row in history.values())}
    print(json.dumps(report, indent=2, ensure_ascii=False))
    return 0 if not errors else 1


if __name__ == '__main__':
    raise SystemExit(main())
