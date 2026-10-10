#!/usr/bin/env python3
"""Fresh 2-family × 2-JDK source/JADX/current-CLI finally replay."""
from __future__ import annotations

import hashlib
import json
import os
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
HERE = Path(__file__).resolve().parent
OUT = HERE / 'finally-final-gates-root-v1'
EVIDENCE = ROOT / 'openspec/evidence/java-syntax-2026-09-22/finally-completion'
BASELINE = ROOT / 'openspec/changes/recover-nested-int-array-compound-updates/results/returned-next-baseline-v1'
BASELINE_MANIFEST = BASELINE / 'manifest.json'
METADATA_DEFAULT = HERE / 'candidate-cli-v2.json'
REMOVED_ENV = ('JAVA_TOOL_OPTIONS', '_JAVA_OPTIONS', 'JDK_JAVA_OPTIONS', 'CLASSPATH')
EXPECTED_PRODUCT_SOURCES = {
    'crates/jarde-java/src/init.rs', 'crates/jarde-java/src/report.rs',
    'crates/jarde-java/src/build.rs', 'crates/jarde-java/src/ast.rs',
    'crates/jarde-java/src/emit.rs', 'crates/jarde-java/src/asserts.rs',
    'crates/jarde-java/src/field.rs', 'src/facade.rs', 'src/class_source.rs', 'Cargo.lock',
}
EXPECTED_TEST_SOURCES = {
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
        'runner_class': 'FinallyNormalRunner',
    },
    'FinallyCompletion': {
        'source': ('openspec/evidence/java-syntax-2026-09-22/finally-completion/FinallyCompletion.java',
                   '543d8bd6252aae4dd072f1d5a4289bd72282b0f94e5782a7e6b066ab0e33404e'),
        'runner': ('openspec/evidence/java-syntax-2026-09-22/finally-completion/FinallyCompletionRunner.java',
                   'b4d12e2b8d80261985baaee0df783cd31a3991855b195d862ae1569c9cec1db5'),
        'jadx': ('openspec/evidence/java-syntax-2026-09-22/finally-completion/jadx.java.txt',
                 '495218a394e5a6652dafc8e54d74dd2e3c5fd58713c77904b89acf7966f5844b'),
        'runner_class': 'FinallyCompletionRunner',
    },
}


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def file_row(path: Path, base: Path) -> dict:
    data = path.read_bytes()
    return {'path': path.relative_to(base).as_posix(), 'bytes': len(data), 'sha256': sha(data)}


def put_bytes(path: Path, data: bytes, base: Path) -> dict:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)
    return {'path': path.relative_to(base).as_posix(), 'bytes': len(data), 'sha256': sha(data)}


def check_binary(row: dict) -> dict:
    path = Path(row['path'])
    data = path.read_bytes()
    if len(data) != row['bytes'] or sha(data) != row['sha256']:
        raise SystemExit(f'frozen JDK tool identity changed: {path}')
    return dict(row)


def run_command(out: Path, label: str, argv: list[str], cwd: Path,
                java_home: str | None, commands: list[dict]) -> dict:
    env = {key: value for key, value in os.environ.items() if key not in REMOVED_ENV}
    if java_home is not None:
        env['JAVA_HOME'] = java_home
        env['PATH'] = str(Path(java_home) / 'bin') + os.pathsep + env.get('PATH', '')
    completed = subprocess.run(argv, cwd=cwd, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
    row = {'label': label, 'argv': argv, 'cwd': str(cwd), 'java_home': java_home,
           'environment_removed': list(REMOVED_ENV), 'exit': completed.returncode}
    for stream, data in (('stdout', completed.stdout), ('stderr', completed.stderr)):
        row[stream] = put_bytes(out / 'streams' / f'{label}.{stream}', data, out)
    commands.append(row)
    return row


def load_json_stream(out: Path, row: dict) -> tuple[dict | None, str | None]:
    try:
        return json.loads((out / row['stdout']['path']).read_bytes()), None
    except (OSError, json.JSONDecodeError, KeyError, TypeError) as error:
        return None, str(error)


def main() -> int:
    if len(sys.argv) != 2 and len(sys.argv) != 3:
        raise SystemExit('usage: replay-finally-final-gates-luna-v1.py FROZEN_CLI [CLI_METADATA_JSON]')
    cli = Path(sys.argv[1]).resolve()
    metadata_path = Path(sys.argv[2]).resolve() if len(sys.argv) == 3 else METADATA_DEFAULT
    metadata_raw = metadata_path.read_bytes()
    metadata = json.loads(metadata_raw)
    if set(metadata.get('candidate_sources', {})) != EXPECTED_PRODUCT_SOURCES:
        raise SystemExit('frozen CLI metadata must include the exact ten production source identities')
    if set(metadata.get('test_sources', {})) != EXPECTED_TEST_SOURCES:
        raise SystemExit('frozen CLI metadata must include the exact five test/workflow source identities')
    if Path(metadata.get('cli_path', '')).resolve() != cli or not cli.is_file() or sha(cli.read_bytes()) != metadata.get('cli_sha256'):
        raise SystemExit('frozen CLI path/hash does not match its metadata')
    for category in ('candidate_sources', 'test_sources'):
        for relative, expected in metadata[category].items():
            current = (ROOT / relative).read_bytes()
            if sha(current) != expected:
                raise SystemExit(f'frozen CLI source identity changed: {relative}')
    if OUT.exists():
        raise SystemExit(f'refusing to overwrite {OUT}')

    baseline_raw = BASELINE_MANIFEST.read_bytes()
    baseline = json.loads(baseline_raw)
    if baseline.get('schema') != 'returned-array-update-next-baseline-v1':
        raise SystemExit('fixed Java tool manifest has unexpected schema')
    tools_by_leg = {}
    for leg, index in (('javac8', 0), ('javac23', 1)):
        tools = baseline['jdk_tools'][index]
        tools_by_leg[leg] = {name: check_binary(row) for name, row in tools.items()}

    validated_inputs = {}
    for family, spec in INPUTS.items():
        validated_inputs[family] = {}
        for kind in ('source', 'runner', 'jadx'):
            relative, expected = spec[kind]
            data = (ROOT / relative).read_bytes()
            if sha(data) != expected:
                raise SystemExit(f'frozen finally input changed: {relative}')
            validated_inputs[family][kind] = data

    OUT.mkdir()
    inputs_manifest = {}
    copied_inputs = {}
    for family, spec in INPUTS.items():
        inputs_manifest[family] = {}
        copied_inputs[family] = {}
        for kind in ('source', 'runner', 'jadx'):
            relative, expected = spec[kind]
            data = validated_inputs[family][kind]
            suffix = '.java' if kind != 'jadx' else '.java'
            name = (family + 'Jadx' + suffix) if kind == 'jadx' else family + suffix if kind == 'source' else spec['runner_class'] + suffix
            saved = put_bytes(OUT / 'inputs' / family / name, data, OUT)
            inputs_manifest[family][kind] = {'source_path': relative, 'sha256': expected, 'copy': saved}
            copied_inputs[family][kind] = OUT / saved['path']

    compiler_flags = ['-source', '8', '-target', '8', '-g:none', '-Xlint:-options']
    commands: list[dict] = []
    cases = []
    expected_cli_count = 0
    for family, spec in INPUTS.items():
        source_path = copied_inputs[family]['source']
        runner_path = copied_inputs[family]['runner']
        jadx_path = copied_inputs[family]['jadx']
        for leg in ('javac8', 'javac23'):
            tools = tools_by_leg[leg]
            java_home = str(Path(tools['java']['path']).parent.parent)
            for kind in ('source', 'jadx', 'jarde'):
                label = f'{family}-{leg}-{kind}'
                case_dir = OUT / 'cases' / label
                case_dir.mkdir(parents=True)
                empty = case_dir / 'empty-classpath-sourcepath'
                empty.mkdir()
                classes = case_dir / 'classes'
                classes.mkdir()
                source_rows = []
                render_record = None
                document = None
                parse_error = None
                package = None
                actual_sources: list[Path] = []
                if kind == 'source':
                    actual_sources = [source_path, runner_path]
                elif kind == 'jadx':
                    jadx_text = jadx_path.read_text()
                    match = re.search(r'^\s*package\s+([\w.]+)\s*;', jadx_text, re.MULTILINE)
                    package = match.group(1) if match else None
                    candidate_source = case_dir / f'{family}.java'
                    candidate_source.write_bytes(jadx_path.read_bytes())
                    runner_target = case_dir / f'{spec["runner_class"]}.java'
                    runner_bytes = runner_path.read_bytes()
                    prefix = f'package {package};\n'.encode() if package else b''
                    runner_target.write_bytes(prefix + runner_bytes)
                    actual_sources = [candidate_source, runner_target]
                else:
                    original_case = next((case for case in cases if case.get('label') == f'{family}-{leg}-source'), None)
                    original_input = None
                    if original_case and original_case.get('compile') and original_case['compile']['exit'] == 0:
                        class_row = next((row for row in original_case.get('candidate_classes', [])
                                          if Path(row['path']).name == f'{family}.class'), None)
                        if class_row:
                            original_input = OUT / 'cases' / original_case['label'] / 'classes' / class_row['path']
                    if original_input is not None and original_input.is_file():
                        copied_class = put_bytes(case_dir / 'input.class', original_input.read_bytes(), OUT)
                        render_argv = [str(cli), 'class-source', '--input', str(case_dir / 'input.class'), '--class',
                                       family, '--policy', 'single-class', '--release', '8', '--format', 'json', '--evidence', 'all']
                        render_record = run_command(OUT, label + '-render', render_argv, ROOT, None, commands)
                        expected_cli_count += 1
                        document, parse_error = load_json_stream(OUT, render_record)
                        if document is not None and isinstance(document.get('text'), str):
                            candidate_source = case_dir / f'{family}.java'
                            candidate_source.write_text(document['text'])
                            actual_sources = [candidate_source, runner_path]
                        else:
                            parse_error = parse_error or 'renderer JSON has no string text field'
                    else:
                        copied_class = None
                        parse_error = 'source/Runner oracle compile did not produce the original class input'

                if actual_sources:
                    compile_argv = [tools['javac']['path'], *compiler_flags,
                                    '-classpath', str(empty), '-sourcepath', str(empty), '-d', str(classes),
                                    *(str(path) for path in actual_sources)]
                    compile_record = run_command(OUT, label + '-compile', compile_argv, ROOT, java_home, commands)
                    source_rows = [file_row(path, OUT) for path in actual_sources]
                    runtime_record = None
                    if compile_record['exit'] == 0:
                        main_class = spec['runner_class']
                        if kind == 'jadx' and package:
                            main_class = package + '.' + main_class
                        runtime_record = run_command(OUT, label + '-run',
                                                     [tools['java']['path'], '-Xverify:all', '-cp', str(classes), main_class],
                                                     ROOT, java_home, commands)
                else:
                    compile_record = None
                    runtime_record = None
                class_rows = [file_row(path, classes) for path in sorted(classes.rglob('*.class'))]
                report_rows = document.get('methods', []) if isinstance(document, dict) else []
                raw_json_row = render_record['stdout'] if render_record else None
                cases.append({
                    'label': label, 'family': family, 'jdk_leg': leg, 'kind': kind,
                    'fresh_jdk_tools': tools, 'java_home': java_home,
                    'original_source_input': inputs_manifest[family]['source']['copy'],
                    'original_runner_input': inputs_manifest[family]['runner']['copy'],
                    'historical_jadx_input': inputs_manifest[family]['jadx']['copy'] if kind == 'jadx' else None,
                    'historical_jadx_freshly_decompiled': False,
                    'jadx_package': package if kind == 'jadx' else None,
                    'original_class_input': copied_class if kind == 'jarde' else None,
                    'render': render_record,
                    'raw_json_stdout': raw_json_row,
                    'report_json_parsed': document is not None if kind == 'jarde' else None,
                    'report_parse_error': parse_error if kind == 'jarde' else None,
                    'report_class': document.get('class') if isinstance(document, dict) else None,
                    'report_text': file_row(case_dir / f'{family}.java', OUT) if kind == 'jarde' and actual_sources else None,
                    'report_members': report_rows,
                    'compile_sources': source_rows,
                    'compiler_flags': compiler_flags if actual_sources else [],
                    'empty_classpath_sourcepath': str(empty),
                    'classes_dir': str(classes),
                    'candidate_classes': class_rows,
                    'compile': compile_record,
                    'runtime': runtime_record,
                    'available': bool(actual_sources),
                    'unavailable_reason': parse_error if not actual_sources else None,
                })

    # Bind each candidate/JADX result to its same-JDK original-source execution oracle.
    by_label = {case['label']: case for case in cases}
    oracle_records = {}
    for family in INPUTS:
        for leg in ('javac8', 'javac23'):
            oracle = by_label[f'{family}-{leg}-source']
            if oracle.get('runtime'):
                oracle_records[f'{family}-{leg}'] = {
                    'exit': oracle['runtime']['exit'], 'stdout': oracle['runtime']['stdout'],
                    'stderr': oracle['runtime']['stderr'], 'source_compile_exit': oracle['compile']['exit'],
                    'source_path': oracle['original_source_input'], 'runner_path': oracle['original_runner_input'],
                    'fresh_execution': True,
                }
            else:
                oracle_records[f'{family}-{leg}'] = {'exit': None, 'stdout': None, 'stderr': None,
                                                     'source_compile_exit': oracle['compile']['exit'] if oracle.get('compile') else None,
                                                     'fresh_execution': True, 'unavailable': True}
            for kind in ('jadx', 'jarde'):
                case = by_label[f'{family}-{leg}-{kind}']
                runtime = case.get('runtime')
                if oracle.get('runtime') is not None and runtime is not None:
                    case['matches_source_oracle_exit_stdout_stderr'] = (
                        runtime['exit'] == oracle['runtime']['exit'] and
                        (OUT / runtime['stdout']['path']).read_bytes() == (OUT / oracle['runtime']['stdout']['path']).read_bytes() and
                        (OUT / runtime['stderr']['path']).read_bytes() == (OUT / oracle['runtime']['stderr']['path']).read_bytes())
                else:
                    case['matches_source_oracle_exit_stdout_stderr'] = None
                case['oracle'] = oracle_records[f'{family}-{leg}']

    unexpected_cli_failures = [command['label'] for command in commands
                               if command['label'].endswith('-render') and command['exit'] != 0]
    runtime_cases = [case for case in cases if case.get('runtime') is not None]
    manifest = {
        'schema': 'finally-final-gates-current-cli-replay-luna-v1',
        'status': 'evidence_recorded',
        'output_root': str(OUT),
        'runner': file_row(Path(__file__).resolve(), HERE),
        'candidate_cli': {'path': str(cli), 'sha256': metadata['cli_sha256'],
                          'metadata_path': str(metadata_path), 'metadata_sha256': sha(metadata_raw),
                          'build_result_sha256': metadata.get('build_result_sha256'),
                          'candidate_sources': metadata['candidate_sources'], 'test_sources': metadata['test_sources']},
        'historical_jdk_manifest': {'path': str(BASELINE_MANIFEST), 'sha256': sha(baseline_raw),
                                    'schema': baseline['schema'], 'jdk_tools': tools_by_leg},
        'frozen_inputs': inputs_manifest,
        'families': list(INPUTS), 'jdk_legs': list(tools_by_leg),
        'kinds': ['source', 'jadx', 'jarde'],
        'expected_case_count': 12, 'actual_case_count': len(cases),
        'expected_cli_render_count': 4, 'actual_cli_render_count': expected_cli_count,
        'unexpected_cli_failures': unexpected_cli_failures,
        'command_count': len(commands), 'commands': commands,
        'source_oracles': oracle_records,
        'cases': cases,
        'actual_counts': {
            'cases': len(cases), 'available_compile_cases': sum(case.get('compile') is not None for case in cases),
            'compile_successes': sum(case.get('compile') is not None and case['compile']['exit'] == 0 for case in cases),
            'runtime_attempts': len(runtime_cases),
            'runtime_successes': sum(case['runtime']['exit'] == 0 for case in runtime_cases),
            'runtime_matches_source_oracle': sum(case.get('matches_source_oracle_exit_stdout_stderr') is True for case in cases),
        },
        'environment_removed': list(REMOVED_ENV),
        'files': [],
    }
    manifest['files'] = [file_row(path, OUT) for path in sorted(OUT.rglob('*'))
                         if path.is_file() and path.name != 'manifest.json']
    (OUT / 'manifest.json').write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + '\n')
    print(json.dumps({'status': manifest['status'], 'output': str(OUT), 'families': len(manifest['families']),
                      'jdk_legs': len(manifest['jdk_legs']), 'cases': len(cases),
                      'cli_renders': expected_cli_count, 'commands': len(commands),
                      'actual_counts': manifest['actual_counts'],
                      'unexpected_cli_failures': unexpected_cli_failures}, ensure_ascii=False))
    return 0 if not unexpected_cli_failures and expected_cli_count == 4 and len(cases) == 12 else 1


if __name__ == '__main__':
    raise SystemExit(main())
