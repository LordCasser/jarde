#!/usr/bin/env python3
"""Replay returned int-array compound updates against frozen bytecode inputs."""
from __future__ import annotations

import hashlib
import importlib.util
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
HERE = Path(__file__).resolve().parent
BASELINE = ROOT / 'openspec/changes/recover-nested-int-array-compound-updates/results/returned-next-baseline-v1'
BASELINE_MANIFEST = BASELINE / 'manifest.json'
FIXTURE = ROOT / 'tests/fixtures/returned-int-array-compound-updates'
REMOVED_ENV = ('JAVA_TOOL_OPTIONS', '_JAVA_OPTIONS', 'JDK_JAVA_OPTIONS', 'CLASSPATH')
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


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def file_row(path: Path, base: Path) -> dict:
    data = path.read_bytes()
    return {'path': path.relative_to(base).as_posix(), 'bytes': len(data), 'sha256': sha(data)}


def copied_row(path: Path, data: bytes, base: Path) -> dict:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)
    return {'path': path.relative_to(base).as_posix(), 'bytes': len(data), 'sha256': sha(data)}


def resolve_record_path(path: str, base: Path) -> Path:
    p = Path(path)
    return p if p.is_absolute() else base / p


def check_row(base: Path, row: dict) -> bytes:
    path = resolve_record_path(row['path'], base)
    data = path.read_bytes()
    if len(data) != row['bytes'] or sha(data) != row['sha256']:
        raise SystemExit(f'hash/size mismatch: {path}')
    return data


def capture(out: Path, label: str, argv: list[str], cwd: Path, java_home: str | None,
            commands: list[dict]) -> tuple[subprocess.CompletedProcess, dict]:
    env = {key: value for key, value in os.environ.items() if key not in REMOVED_ENV}
    if java_home:
        home = Path(java_home)
        env['JAVA_HOME'] = str(home)
        env['PATH'] = str(home / 'bin') + os.pathsep + env.get('PATH', '')
    result = subprocess.run(argv, cwd=cwd, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
    row = {'label': label, 'argv': argv, 'cwd': str(cwd), 'java_home': java_home,
           'environment_removed': list(REMOVED_ENV), 'exit': result.returncode}
    for stream, raw in (('stdout', result.stdout), ('stderr', result.stderr)):
        row[stream] = copied_row(out / 'streams' / f'{label}.{stream}', raw, out)
    commands.append(row)
    return result, row


def load_physical_parser():
    parser_path = ROOT / 'openspec/changes/recover-covariant-child-array-initializers/results/prepare-child-jvm-controls-v1.py'
    sys.dont_write_bytecode = True
    spec = importlib.util.spec_from_file_location('returned_candidate_physical_parser', parser_path)
    if spec is None or spec.loader is None:
        raise SystemExit(f'cannot load read-only class parser: {parser_path}')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def decoded_members(document: dict) -> list[dict]:
    rows = []
    for method in document.get('methods', []):
        item = method.get('item', {})
        outcome = method.get('outcome', {})
        report = outcome.get('report') if outcome.get('kind') == 'recovered' else None
        source_map = report.get('source_map') if report else None
        bcis = set()
        for segment in (source_map or {}).get('segments', []):
            origin = segment.get('origin', {})
            primary = origin.get('primary', {})
            if isinstance(primary.get('bci'), int):
                bcis.add(primary['bci'])
            for derived in origin.get('derived', []):
                if isinstance(derived.get('bci'), int):
                    bcis.add(derived['bci'])
        body = report.get('text', '') if report else ''
        rows.append({
            'name': item.get('name', {}).get('escaped'),
            'name_raw': item.get('name', {}).get('raw'),
            'descriptor': item.get('descriptor', {}).get('escaped'),
            'descriptor_raw': item.get('descriptor', {}).get('raw'),
            'access_flags': item.get('access_flags'), 'declaration': method.get('declaration'),
            'outcome': outcome.get('kind'), 'refusal': outcome.get('refusal'),
            'quality': report.get('quality') if report else None,
            'representation': report.get('representation') if report else None,
            'markers': method.get('markers', []), 'body': body,
            'body_sha256': sha(body.encode()) if report else None,
            'source_map': source_map, 'source_map_all_bcis': sorted(bcis),
        })
    return rows


def main() -> int:
    if len(sys.argv) != 4:
        raise SystemExit('usage: replay-returned-candidate-luna-v1.py FROZEN_CLI CLI_METADATA_JSON OUTPUT_DIR')
    cli, metadata_path, out = (Path(arg).resolve() for arg in sys.argv[1:])
    metadata_raw = metadata_path.read_bytes()
    metadata = json.loads(metadata_raw)
    if not cli.is_file() or sha(cli.read_bytes()) != metadata.get('cli_sha256') or Path(metadata.get('cli_path', '')).resolve() != cli:
        raise SystemExit('candidate CLI identity mismatch')
    if set(metadata.get('candidate_sources', {})) != PRODUCTS or set(metadata.get('test_sources', {})) != TESTS:
        raise SystemExit('candidate metadata source sets changed')
    for relative, expected in {**metadata['candidate_sources'], **metadata['test_sources']}.items():
        if sha((ROOT / relative).read_bytes()) != expected:
            raise SystemExit(f'frozen candidate source changed: {relative}')
    canonical = metadata.get('canonical_files', {})
    if not canonical:
        raise SystemExit('candidate metadata has no canonical fixture inventory')
    for relative, expected in canonical.items():
        if sha((ROOT / relative).read_bytes()) != expected:
            raise SystemExit(f'canonical fixture changed: {relative}')
    if out.exists():
        raise SystemExit(f'refusing to overwrite {out}')

    baseline_raw = BASELINE_MANIFEST.read_bytes()
    baseline = json.loads(baseline_raw)
    if baseline.get('schema') != 'returned-array-update-next-baseline-v1' or baseline.get('scope') != 'next-candidate analysis only; no product/spec implementation':
        raise SystemExit('historical original/JADX baseline has unexpected identity or scope')
    for row in baseline['files']:
        check_row(BASELINE, row)
    if sha((FIXTURE / 'ReturnedIntArrayUpdates.java').read_bytes()) != canonical['tests/fixtures/returned-int-array-compound-updates/ReturnedIntArrayUpdates.java']:
        raise SystemExit('fixture source differs from frozen canonical identity')
    if sha((FIXTURE / 'Runner.java').read_bytes()) != canonical['tests/fixtures/returned-int-array-compound-updates/Runner.java']:
        raise SystemExit('fixture Runner differs from frozen canonical identity')

    output = FIXTURE / 'expected.stdout'
    expected_stdout = output.read_bytes()
    expected_stderr = (FIXTURE / 'expected.stderr').read_bytes()
    if expected_stderr != b'':
        raise SystemExit('fixture stderr oracle is not empty')
    baseline_cases = {case['label']: case for case in baseline['cases']}
    tools_by_leg = {f'javac{version}': tools for version, tools in zip((8, 23), baseline['jdk_tools'])}
    historical_cases = []
    for label, case in baseline_cases.items():
        if case.get('kind') not in ('original', 'jadx'):
            continue
        refs = {'label': label, 'kind': case['kind'], 'success': case['success'], 'fresh_execution': False,
                'sources': case['sources'], 'classes': case['classes'], 'compile': case['compile'],
                'runtime': case['runtime'], 'decompile': case.get('decompile'),
                'profile': case.get('profile'), 'runner_package_prefix': case.get('runner_package_prefix')}
        historical_cases.append(refs)
    if len([x for x in historical_cases if x['kind'] == 'original']) != 2 or len([x for x in historical_cases if x['kind'] == 'jadx']) != 4:
        raise SystemExit('historical baseline does not contain exact original/JADX case set')

    out.mkdir(parents=True)
    fixture_source = copied_row(out / 'inputs' / 'ReturnedIntArrayUpdates.java', (FIXTURE / 'ReturnedIntArrayUpdates.java').read_bytes(), out)
    fixture_runner = copied_row(out / 'inputs' / 'Runner.java', (FIXTURE / 'Runner.java').read_bytes(), out)
    oracle_stdout = copied_row(out / 'oracle' / 'expected.stdout', expected_stdout, out)
    oracle_stderr = copied_row(out / 'oracle' / 'expected.stderr', expected_stderr, out)
    commands: list[dict] = []
    input_rows = []
    cases = []
    parser = load_physical_parser()
    for leg in ('javac8', 'javac23'):
        tools = tools_by_leg[leg]
        for name, row in tools.items():
            check_row(Path('/'), row)
        original_case = baseline_cases[f'{leg}-original']
        original_class_row = next(row for row in original_case['classes'] if Path(row['path']).name == 'ReturnedIntArrayUpdates.class')
        original_class = check_row(BASELINE, original_class_row)
        case_label = f'{leg}-jarde'
        case_dir = out / 'cases' / case_label
        case_dir.mkdir(parents=True)
        input_record = copied_row(case_dir / 'input.class', original_class, out)
        input_rows.append({'case': case_label, 'historical_source_path': original_class_row['path'],
                           'historical_source_sha256': original_class_row['sha256'], 'copied_input': input_record,
                           'original_raw_fresh': False})
        original_run = original_case['runtime']
        original_stdout = check_row(BASELINE, original_run['stdout'])
        original_stderr = check_row(BASELINE, original_run['stderr'])
        if original_stdout != expected_stdout or original_stderr != expected_stderr or original_run['exit'] != 0:
            raise SystemExit(f'{leg}: historical original run differs from fixture oracle')
        original = {'exit': original_run['exit'], 'stdout': copied_row(case_dir / 'original.stdout', original_stdout, out),
                    'stderr': copied_row(case_dir / 'original.stderr', original_stderr, out), 'fresh_execution': False}

        render_argv = [str(cli), 'class-source', '--input', str(case_dir / 'input.class'), '--class',
                       'ReturnedIntArrayUpdates', '--policy', 'single-class', '--release', '8', '--format', 'json', '--evidence', 'all']
        rendered, render_record = capture(out, case_label + '-render', render_argv, ROOT, None, commands)
        document = None
        parse_error = None
        generated_source = None
        if rendered.returncode == 0:
            try:
                document = json.loads(rendered.stdout)
                source_text = document['text']
                generated_source = case_dir / 'ReturnedIntArrayUpdates.java'
                generated_source.write_text(source_text)
            except (json.JSONDecodeError, KeyError, TypeError) as error:
                parse_error = str(error)
        member_rows = decoded_members(document) if document else []
        report = {'class': document.get('class') if document else None, 'json_parsed': document is not None,
                  'source': file_row(generated_source, out) if generated_source else None,
                  'source_text_sha256': sha(document['text'].encode()) if document else None,
                  'members': member_rows, 'parse_error': parse_error, 'raw_json_stdout': render_record['stdout']}
        compile_record = runtime_record = None
        compiled_sources = []
        classes_dir = case_dir / 'classes'
        empty = case_dir / 'empty-classpath-sourcepath'
        if generated_source:
            runner_copy = case_dir / 'Runner.java'
            shutil.copyfile(FIXTURE / 'Runner.java', runner_copy)
            compiled_sources = [generated_source, runner_copy]
            empty.mkdir()
            classes_dir.mkdir()
            compiler_flags = ['-source', '8', '-target', '8', '-g:none', '-Xlint:-options']
            java_home = str(Path(tools['java']['path']).parent.parent)
            compile_argv = [tools['javac']['path'], *compiler_flags, '-classpath', str(empty), '-sourcepath', str(empty),
                            '-d', str(classes_dir), *(str(path) for path in compiled_sources)]
            _, compile_record = capture(out, case_label + '-compile', compile_argv, ROOT, java_home, commands)
            if compile_record['exit'] == 0:
                run_argv = [tools['java']['path'], '-Xverify:all', '-cp', str(classes_dir), 'Runner']
                _, runtime_record = capture(out, case_label + '-run', run_argv, ROOT, java_home, commands)
        class_rows = [file_row(path, classes_dir) for path in sorted(classes_dir.rglob('*.class'))] if classes_dir.exists() else []
        runtime_matches = bool(runtime_record and runtime_record['exit'] == original['exit'] and
                               (out / runtime_record['stdout']['path']).read_bytes() == original_stdout and
                               (out / runtime_record['stderr']['path']).read_bytes() == original_stderr)
        success = bool(report['json_parsed'] and len(member_rows) == 11 and compile_record and compile_record['exit'] == 0 and
                       runtime_record and runtime_matches)
        cases.append({'label': case_label, 'leg': leg, 'input': input_record, 'original': original,
                      'original_stream_source': {'manifest': str(BASELINE_MANIFEST), 'command_label': original_run['label'],
                                                 'stdout': original_run['stdout'], 'stderr': original_run['stderr'],
                                                 'fresh_execution': False},
                      'report': report, 'compile_sources': [file_row(path, out) for path in compiled_sources],
                      'compiler_flags': compiler_flags if generated_source else [],
                      'empty_classpath_sourcepath': str(empty), 'candidate_classes': class_rows,
                      'compile': compile_record, 'runtime': runtime_record,
                      'runtime_matches_original_exit_stdout_stderr': runtime_matches, 'candidate_success': success})

    # Record physical class facts used to interpret every positive source-map anchor.
    physical = []
    for case in cases:
        raw = (out / case['input']['path']).read_bytes()
        parsed = parser.parse_class(raw)
        members = {row['name']: row for row in case['report']['members']}
        method_facts = []
        for name in COMPOUND:
            descriptor = members[name]['descriptor']
            code = parser.find_method_code(raw, parsed, name, descriptor)
            instructions = parser.decode_instructions(raw, code)
            physical_rows = [{'bci': ins['bci'], 'opcode': ins['opcode'], 'mnemonic': ins.get('mnemonic')} for ins in instructions]
            method_facts.append({'name': name, 'descriptor': descriptor, 'instructions': physical_rows})
        physical.append({'case': case['label'], 'methods': method_facts})

    historical_refs = [{'path': str(BASELINE_MANIFEST), 'sha256': sha(baseline_raw), 'fresh_execution_claim': False,
                        'scope': baseline['scope']}]
    for row in baseline['files']:
        historical_refs.append({'path': str(BASELINE / row['path']), 'sha256': row['sha256'],
                                'bytes': row['bytes'], 'fresh_execution_claim': False})
    manifest = {
        'schema': 'returned-int-array-compound-candidate-luna-v1', 'status': 'complete_replay_recorded',
        'runner': file_row(Path(__file__).resolve(), HERE), 'output_root': str(out),
        'candidate_cli': {'path': str(cli), 'sha256': metadata['cli_sha256'], 'metadata_path': str(metadata_path),
                          'metadata_sha256': sha(metadata_raw), 'candidate_sources': metadata['candidate_sources'],
                          'test_sources': metadata['test_sources'], 'canonical_files': canonical},
        'historical_baseline': historical_refs[0], 'historical_cases': historical_cases,
        'fixture_inputs': {'source': fixture_source, 'runner': fixture_runner,
                           'expected_stdout': oracle_stdout, 'expected_stderr': oracle_stderr},
        'input_classes': input_rows, 'commands': commands, 'cases': cases,
        'physical_methods': physical, 'expected_denominator': 2, 'environment_removed': list(REMOVED_ENV),
        'files': [],
    }
    manifest['files'] = [file_row(path, out) for path in sorted(out.rglob('*')) if path.is_file() and path.name != 'manifest.json']
    (out / 'manifest.json').write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + '\n')
    if len(cases) != 2 or not all(case['candidate_success'] for case in cases):
        raise SystemExit('returned candidate replay failed; raw evidence retained in manifest')
    print(json.dumps({'status': manifest['status'], 'cases': len(cases),
                      'successes': sum(case['candidate_success'] for case in cases),
                      'manifest': str(out / 'manifest.json')}))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
