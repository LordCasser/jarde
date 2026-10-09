#!/usr/bin/env python3
"""Replay the frozen nested-array control against old Jarde and fresh JADX baselines.

Root-only execution script. It does not alter the canonical fixture and refuses to overwrite
controls-baseline-v1. Jarde and JADX output is rendered anew from each frozen
NestedIntUpdates.class file; the fixed Runner source remains an external behavioral harness.
"""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess

ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
RESULTS = ROOT / 'openspec/changes/recover-nested-int-array-compound-updates/results'
OUT = RESULTS / 'controls-baseline-v1'
CONTROLS_ROOT = RESULTS / 'controls-v1'
CONTROLS_MANIFEST = CONTROLS_ROOT / 'manifest.json'
CLI_METADATA_PATH = ROOT / 'openspec/changes/recover-bigdecimal-number-widening/results/candidate-cli-v1.json'
EXPECTED_CLI_SHA = 'b79520629443a0211cd656d1374e415f76ac6adf400d79cb86154954caba9cb0'
JADX_BASELINE_PATH = ROOT / 'openspec/changes/recover-bigdecimal-number-widening/results/nested-array-jadx-baseline-v2/manifest.json'
EXPECTED_JADX_SCHEMA = 'nested-array-fresh-jadx-baseline-v2'
ENV_KEYS_REMOVED = ('JAVA_TOOL_OPTIONS', '_JAVA_OPTIONS', 'JDK_JAVA_OPTIONS', 'CLASSPATH')
JADX_BASELINE_CASES = 4


def sha_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha_file(path: Path) -> str:
    return sha_bytes(path.read_bytes())


def file_record(path: Path, base: Path = ROOT) -> dict:
    return {'path': str(path.relative_to(base)), 'bytes': path.stat().st_size, 'sha256': sha_file(path)}


def stream_record(path: Path, data: bytes) -> dict:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)
    return {'path': str(path.relative_to(OUT)), 'bytes': len(data), 'sha256': sha_bytes(data)}


def clean_environment(java_home: str | None = None) -> tuple[dict[str, str], dict[str, bool]]:
    inherited = {key: key in os.environ for key in ENV_KEYS_REMOVED}
    env = {key: value for key, value in os.environ.items() if key not in ENV_KEYS_REMOVED}
    if java_home is not None:
        env['JAVA_HOME'] = java_home
        env['PATH'] = str(Path(java_home) / 'bin') + os.pathsep + env.get('PATH', '')
    return env, inherited


def execute(label: str, argv: list[str], cwd: Path, env: dict[str, str], java_home: str | None,
            commands: list[dict]) -> tuple[subprocess.CompletedProcess, dict]:
    result = subprocess.run(argv, cwd=cwd, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                            check=False)
    stdout = stream_record(OUT / 'streams' / f'{label}.stdout', result.stdout)
    stderr = stream_record(OUT / 'streams' / f'{label}.stderr', result.stderr)
    row = {'label': label, 'argv': argv, 'cwd': str(cwd), 'exit': result.returncode,
           'environment_removed': list(ENV_KEYS_REMOVED), 'java_home': java_home,
           'stdout': stdout, 'stderr': stderr}
    commands.append(row)
    return result, row


def verify_closed_manifest(root: Path, manifest: dict) -> None:
    rows = manifest.get('files', [])
    for row in rows:
        path = root / row['path']
        if not path.is_file() or path.stat().st_size != row['bytes'] or sha_file(path) != row['sha256']:
            raise SystemExit(f'frozen JADX distribution artifact changed: {path}')


def source_package(text: str) -> str:
    match = re.search(r'^\s*package\s+([A-Za-z_$][\w$]*(?:\.[A-Za-z_$][\w$]*)*)\s*;', text, re.M)
    return match.group(1) if match else ''


def summarize_members(document: dict) -> list[dict]:
    summaries = []
    for method in document.get('methods', []):
        item = method.get('item', {})
        name = item.get('name', {})
        descriptor = item.get('descriptor', {})
        outcome = method.get('outcome', {})
        report = outcome.get('report') if outcome.get('kind') == 'recovered' else None
        body = report.get('text', '') if report else ''
        source_map = report.get('source_map') if report else None
        summaries.append({
            'name': name.get('escaped'), 'name_raw': name.get('raw'),
            'descriptor': descriptor.get('escaped'), 'descriptor_raw': descriptor.get('raw'),
            'access_flags': item.get('access_flags'), 'declaration': method.get('declaration'),
            'outcome_kind': outcome.get('kind'), 'refusal': outcome.get('refusal'),
            'quality': report.get('quality') if report else None,
            'representation': report.get('representation') if report else None,
            'markers': method.get('markers', []), 'body': body,
            'body_sha256': sha_bytes(body.encode()) if report else None,
            'source_map': source_map,
            'source_map_sha256': sha_bytes(json.dumps(source_map, sort_keys=True,
                                                       separators=(',', ':')).encode()) if source_map is not None else None,
        })
    return summaries


def render_sources(cli: Path, class_file: Path, class_names: list[str], label: str,
                   commands: list[dict], env: dict[str, str], reports_root: Path) -> list[dict]:
    reports = []
    for class_name in class_names:
        command_label = f'{label}-render-{class_name.replace(".", "_")}'
        argv = [str(cli), 'class-source', '--input', str(class_file), '--class', class_name,
                '--policy', 'single-class', '--format', 'json', '--evidence', 'all', '--release', '8']
        result, command = execute(command_label, argv, ROOT, env, None, commands)
        row = {'class': class_name, 'command': command, 'json_parsed': False,
               'source_path': None, 'source_sha256': None, 'members': [], 'parse_error': None}
        if result.returncode == 0:
            try:
                document = json.loads(result.stdout)
                text = document['text']
                target = reports_root / 'rendered' / Path(*class_name.split('.')).with_suffix('.java')
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_text(text, encoding='utf-8')
                row.update({'json_parsed': True, 'source_path': str(target.relative_to(OUT)),
                            'source_sha256': sha_bytes(text.encode()), 'class_declaration': document.get('declaration'),
                            'coverage': document.get('coverage'), 'diagnostics': document.get('diagnostics'),
                            'fields': document.get('fields', []), 'members': summarize_members(document)})
            except (json.JSONDecodeError, KeyError, TypeError, OSError) as error:
                row['parse_error'] = str(error)
        reports.append(row)
    return reports


def copy_input_class(leg: str, control_leg: dict, out: Path) -> tuple[Path, dict]:
    expected = {Path(row['path']).name: row for row in control_leg['class_files']}
    if set(expected) != {'NestedIntUpdates.class', 'Runner.class'}:
        raise SystemExit(f'{leg}: control class closure changed: {sorted(expected)}')
    row = expected['NestedIntUpdates.class']
    source = ROOT / row['path']
    if not source.is_file() or source.stat().st_size != row['bytes'] or sha_file(source) != row['sha256']:
        raise SystemExit(f'{leg}: frozen original class identity changed: {source}')
    dest = out / 'input-classes' / leg / 'NestedIntUpdates.class'
    dest.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(source, dest)
    copied = file_record(dest)
    return dest, {'original_class': row, 'copied_class': copied,
                  'input_class_closure_in_controls': control_leg['class_files']}


def original_oracle(control: dict, leg: str, out: Path) -> dict:
    row = next(r for r in control['legs'] if r['leg'] == leg)
    run = next(c for c in row['commands'] if c['label'] == 'run-runner')
    streams = {}
    for key in ('stdout', 'stderr'):
        source = CONTROLS_ROOT / run[key]['path']
        data = source.read_bytes()
        if len(data) != run[key]['bytes'] or sha_bytes(data) != run[key]['sha256']:
            raise SystemExit(f'{leg}: original Runner {key} disagrees with frozen manifest')
        dest = out / 'original-oracle' / f'{leg}.{key}'
        record = stream_record(dest, data)
        streams[key] = record
    return {'exit': run['exit'], 'stdout': streams['stdout'], 'stderr': streams['stderr'],
            'fresh_execution': False, 'source_command': run['label']}


def compare_runtime(runtime: dict | None, original: dict, expected_cases: list[dict]) -> dict:
    if runtime is None:
        return {'available': False, 'exit_match': False, 'stdout_match': False,
                'stderr_match': False, 'matches': False, 'case_lines': []}
    runtime_stdout = (OUT / runtime['stdout']['path']).read_bytes()
    runtime_stderr = (OUT / runtime['stderr']['path']).read_bytes()
    original_stdout = (OUT / original['stdout']['path']).read_bytes()
    original_stderr = (OUT / original['stderr']['path']).read_bytes()
    lines = runtime_stdout.decode('utf-8', errors='replace').splitlines()
    case_lines = []
    for index, case in enumerate(expected_cases):
        actual = lines[index] if index < len(lines) else None
        case_lines.append({'case': case['name'], 'expected_line': case['expected'],
                           'actual_line': actual, 'matches': actual == case['expected']})
    exit_match = runtime['exit'] == original['exit']
    stdout_match = runtime_stdout == original_stdout
    stderr_match = runtime_stderr == original_stderr
    return {'available': True, 'exit_match': exit_match, 'stdout_match': stdout_match,
            'stderr_match': stderr_match, 'matches': exit_match and stdout_match and stderr_match,
            'case_lines': case_lines, 'candidate_stdout_sha256': sha_bytes(runtime_stdout),
            'candidate_stderr_sha256': sha_bytes(runtime_stderr),
            'original_stdout_sha256': sha_bytes(original_stdout),
            'original_stderr_sha256': sha_bytes(original_stderr)}


def prepare_compile_sources(source_paths: list[Path], frozen_runner: Path, dest: Path,
                            target_text: str) -> tuple[list[Path], dict]:
    package = source_package(target_text)
    original_runner = frozen_runner.read_bytes()
    runner_text = original_runner.decode('utf-8')
    runner_package = source_package(runner_text)
    if runner_package:
        raise SystemExit('frozen Runner unexpectedly has a package declaration')
    adapted_text = (f'package {package};\n' if package else '') + runner_text
    adapted_body = (re.sub(r'^package\s+[A-Za-z_$][\w$]*(?:\.[A-Za-z_$][\w$]*)*\s*;\s*\n',
                           '', adapted_text, count=1) if package else adapted_text)
    if adapted_body != runner_text:
        raise SystemExit('Runner adaptation changed bytes beyond the required package declaration')
    dest.mkdir(parents=True, exist_ok=False)
    adapted = dest / 'Runner.java'
    adapted.write_text(adapted_text, encoding='utf-8')
    copied = []
    for source in source_paths:
        target = dest / source.name
        if target.name == 'Runner.java':
            continue
        shutil.copyfile(source, target)
        copied.append(target)
    copied.append(adapted)
    metadata = {'package': package or None, 'runner_original_sha256': sha_bytes(original_runner),
                'runner_original_body_sha256': sha_bytes(runner_text.encode()),
                'runner_adapted_body_sha256': sha_bytes(adapted_body.encode()),
                'runner_body_preserved': True,
                'runner_adapted_sha256': sha_bytes(adapted_text.encode()),
                'runner_adaptation': 'prepend matching package declaration only' if package else 'byte-identical frozen Runner source',
                'compiled_source_paths': [str(path) for path in copied]}
    return copied, metadata


def execute_candidate_case(label: str, kind: str, leg: str, input_path: Path,
                           class_names: list[str], cli: Path | None, jadx: Path | None,
                           profile: str | None, control_leg: dict, control_manifest: dict,
                           original: dict,
                           commands: list[dict], env: dict[str, str], java_home: str,
                           jadx_java_home: str,
                           expected_cases: list[dict]) -> dict:
    root = OUT / 'cases' / label
    root.mkdir(parents=True)
    javac = control_leg['jdk_tools']['javac']['path']
    java = control_leg['jdk_tools']['java']['path']
    leg_jdk_home = str(Path(java).parent.parent)
    empty = root / 'empty-classpath-sourcepath'
    classes = root / 'classes'
    empty.mkdir()
    classes.mkdir()
    report_root = root / 'reports'
    reports = []
    decompile_command = None
    decompile_exit = None
    source_paths = []
    source_complete = False
    target_source = None
    jadx_package = None

    if kind == 'jarde':
        reports = render_sources(cli, input_path, class_names, label, commands, env, report_root)
        source_rows = [row for row in reports if row.get('class') == 'NestedIntUpdates' and row.get('source_path')]
        target_source = OUT / source_rows[0]['source_path'] if len(source_rows) == 1 else None
        runner_source = Path(control_manifest['fixture_root']) / 'Runner.java'
        all_reported = (len(reports) == 1 and reports[0].get('class') == 'NestedIntUpdates'
                        and all(row.get('json_parsed') for row in reports))
        if target_source and runner_source.is_file():
            compile_sources_dir = root / 'compile-sources'
            sources, runner_adaptation = prepare_compile_sources([target_source], runner_source,
                                                                  compile_sources_dir,
                                                                  target_source.read_text())
            runner_adaptation['runner_is_external_behavioral_harness'] = True
            source_complete = all_reported
        else:
            sources, runner_adaptation = [], {'error': 'missing target source or frozen Runner.java'}
    else:
        profile_root = root / 'jadx-output'
        flags = [str(jadx), '--no-res', '--config', 'none', '--threads-count', '1']
        if profile == 'none':
            flags.extend(['--rename-flags', 'none'])
        flags.extend(['-d', str(profile_root), str(input_path)])
        jadx_env, _ = clean_environment(jadx_java_home)
        result, decompile_command = execute(f'{label}-decompile', flags, ROOT, jadx_env,
                                            jadx_java_home, commands)
        decompile_exit = result.returncode
        generated = sorted(profile_root.rglob('*.java'))
        generated_records = []
        for source in generated:
            generated_records.append(file_record(source, OUT))
        target_matches = [source for source in generated
                          if re.search(r'\b(?:class|interface|enum)\s+NestedIntUpdates\b',
                                       source.read_text(encoding='utf-8'))]
        target_source = target_matches[0] if len(target_matches) == 1 else None
        if target_source:
            target_text = target_source.read_text(encoding='utf-8')
            jadx_package = source_package(target_text)
        runner_source = Path(control_manifest['fixture_root']) / 'Runner.java'
        if result.returncode == 0 and generated and target_source and runner_source.is_file():
            compile_sources_dir = root / 'compile-sources'
            sources, runner_adaptation = prepare_compile_sources(generated, runner_source,
                                                                  compile_sources_dir,
                                                                  target_source.read_text(encoding='utf-8'))
            source_complete = len(target_matches) == 1 and len(generated) == 1
        else:
            sources, runner_adaptation = [], {'error': 'decompile failed or full target source set/Runner unavailable'}
        reports = [{'class': source.stem, 'source_path': str(source.relative_to(OUT)),
                    'source_sha256': sha_file(source), 'generated_by': 'JADX',
                    'package': source_package(source.read_text(encoding='utf-8')),
                    'bytes': source.stat().st_size} for source in generated]

    compile_record = None
    compile_exit = None
    runtime_record = None
    runtime_comparison = None
    compile_flags = None
    source_records = []
    if sources:
        original_compile = next(c for c in control_leg['commands'] if c['label'] == 'compile')
        args = original_compile['argv']
        cp_index = args.index('-classpath')
        compile_flags = args[1:cp_index]
        compile_argv = [javac, *compile_flags, '-classpath', str(empty), '-sourcepath', str(empty),
                        '-d', str(classes), *[str(p) for p in sorted(sources)]]
        leg_env, _ = clean_environment(leg_jdk_home)
        compile_result, compile_record = execute(f'{label}-compile', compile_argv, ROOT,
                                                 leg_env, leg_jdk_home, commands)
        compile_exit = compile_result.returncode
        source_records = [file_record(path) for path in sorted(sources)]
        if compile_exit == 0:
            fqcn = f'{runner_adaptation.get("package")}.Runner' if runner_adaptation.get('package') else 'Runner'
            runtime_argv = [java, '-Xverify:all', '-cp', str(classes), fqcn]
            _, runtime_record = execute(f'{label}-run', runtime_argv, ROOT,
                                        leg_env, leg_jdk_home, commands)
            runtime_comparison = compare_runtime(runtime_record, original, control_manifest['cases'])

    generated_classes = sorted(classes.rglob('*.class'))
    return {
        'label': label, 'kind': kind, 'leg': leg, 'rename_profile': profile,
        'input_artifact': str(input_path.relative_to(OUT)),
        'input_artifact_kind': 'frozen-original-classfile',
        'input_class_names': class_names,
        'decompile_command': decompile_command, 'decompile_exit': decompile_exit,
        'source_set_complete': source_complete, 'reports': reports,
        'target_source_path': str(target_source.relative_to(OUT)) if target_source else None,
        'jadx_package': jadx_package, 'runner_adaptation': runner_adaptation,
        'compile_flags_reused_from_controls': compile_flags, 'compiled_sources': source_records,
        'candidate_compile': compile_record, 'compile_exit': compile_exit,
        'candidate_class_set': [file_record(path, classes) for path in generated_classes],
        'candidate_runtime': runtime_record, 'original_runtime': original,
        'runtime_comparison': runtime_comparison,
    }


def main() -> None:
    if OUT.exists():
        raise SystemExit(f'refusing to overwrite baseline output directory: {OUT}')
    if not CONTROLS_MANIFEST.is_file():
        raise SystemExit(f'missing frozen controls manifest: {CONTROLS_MANIFEST}')
    control_raw = CONTROLS_MANIFEST.read_bytes()
    control = json.loads(control_raw)
    if control.get('schema') != 'nested-int-array-compound-controls-v1' or control.get('status') != 'complete':
        raise SystemExit('controls-v1 must be complete before replay')
    control_hash = sha_bytes(control_raw)
    for file_row in control['closed_inventory']:
        path = ROOT / file_row['path']
        if not path.is_file() or path.stat().st_size != file_row['bytes'] or sha_file(path) != file_row['sha256']:
            raise SystemExit(f'controls-v1 frozen input changed: {path}')
    controls_runner_path = ROOT / control['runner']['path']
    if (not controls_runner_path.is_file() or controls_runner_path.stat().st_size != control['runner']['bytes']
            or sha_file(controls_runner_path) != control['runner']['sha256']):
        raise SystemExit('controls-v1 runner identity changed')
    toolchain_path = Path(control['toolchain_manifest']['path'])
    toolchain_bytes = toolchain_path.read_bytes()
    if sha_bytes(toolchain_bytes) != control['toolchain_manifest']['sha256']:
        raise SystemExit('frozen toolchain manifest identity changed')
    toolchain_manifest = json.loads(toolchain_bytes)
    frozen_legs = {row['leg']: row for row in toolchain_manifest.get('legs', [])}
    if set(frozen_legs) != {'javac8', 'javac23'}:
        raise SystemExit('frozen toolchain manifest leg set changed')
    for leg, control_leg in ((row['leg'], row) for row in control['legs']):
        for tool in ('javac', 'java', 'javap'):
            identity = control_leg['jdk_tools'][tool]
            expected = frozen_legs[leg]['tools'][tool]
            if identity != expected:
                raise SystemExit(f'{leg}: controls tool identity differs from frozen JDK manifest for {tool}')
            path = Path(identity['path'])
            if not path.is_file() or path.stat().st_size != identity['bytes'] or sha_file(path) != identity['sha256']:
                raise SystemExit(f'{leg}: frozen {tool} identity changed: {path}')

    cli_metadata_raw = CLI_METADATA_PATH.read_bytes()
    cli_metadata = json.loads(cli_metadata_raw)
    cli_path = Path(cli_metadata['cli_path'])
    if cli_metadata['cli_sha256'] != EXPECTED_CLI_SHA or not cli_path.is_file() or sha_file(cli_path) != EXPECTED_CLI_SHA:
        raise SystemExit('frozen BigDecimal candidate CLI identity mismatch')
    for relative, expected in cli_metadata.get('candidate_sources', {}).items():
        current = ROOT / relative
        if not current.is_file() or sha_file(current) != expected:
            raise SystemExit(f'frozen old-CLI product source changed: {current}')
    jadx_manifest_raw = JADX_BASELINE_PATH.read_bytes()
    jadx_manifest = json.loads(jadx_manifest_raw)
    if jadx_manifest.get('schema') != EXPECTED_JADX_SCHEMA:
        raise SystemExit('unexpected JADX distribution identity manifest')
    verify_closed_manifest(ROOT, {'files': jadx_manifest['jadx_distribution_files']})
    jadx_record = jadx_manifest['jadx_wrapper']
    jadx_path = Path(jadx_record['path'])
    if not jadx_path.is_file() or jadx_path.stat().st_size != jadx_record['bytes'] or sha_file(jadx_path) != jadx_record['sha256']:
        raise SystemExit('JADX launcher identity mismatch')
    if jadx_manifest.get('version_stdout') != '1.5.6\n' or len(jadx_manifest['cases']) != JADX_BASELINE_CASES:
        raise SystemExit('fresh JADX 1.5.6 baseline identity or four-case denominator changed')

    OUT.mkdir()
    (OUT / 'logs').mkdir()
    commands: list[dict] = []
    inherited = {key: key in os.environ for key in ENV_KEYS_REMOVED}
    legs = {row['leg']: row for row in control['legs']}
    if set(legs) != {'javac8', 'javac23'}:
        raise SystemExit('controls-v1 must contain javac8 and javac23')
    frozen_runner = Path(control['fixture_root']) / 'Runner.java'
    expected_sources = {row['path']: row['sha256'] for row in control['source_files']}
    if set(expected_sources) != {
            'tests/fixtures/nested-int-array-compound-updates/NestedIntUpdates.java',
            'tests/fixtures/nested-int-array-compound-updates/Runner.java'}:
        raise SystemExit('canonical control source closure changed')
    for relative, expected in expected_sources.items():
        if sha_file(ROOT / relative) != expected:
            raise SystemExit(f'canonical controls source changed: {relative}')

    originals = {leg: original_oracle(control, leg, OUT) for leg in ('javac8', 'javac23')}

    jdk23_home = str(Path(legs['javac23']['jdk_tools']['java']['path']).parent.parent)
    jadx_env, _ = clean_environment(jdk23_home)
    version, version_record = execute('jadx-version', [str(jadx_path), '--version'], ROOT,
                                     jadx_env, jdk23_home, commands)
    if version.returncode != 0 or version.stdout != b'1.5.6\n':
        raise SystemExit('fresh JADX launcher did not report the pinned 1.5.6 version')

    cases = []
    input_records = []
    for leg in ('javac8', 'javac23'):
        control_leg = legs[leg]
        input_class, input_record = copy_input_class(leg, control_leg, OUT)
        input_records.append({'leg': leg, **input_record})
        env, _ = clean_environment(str(Path(control_leg['jdk_tools']['java']['path']).parent.parent))
        cases.append(execute_candidate_case(f'jarde-{leg}', 'jarde', leg, input_class, ['NestedIntUpdates'],
                                           cli_path, None, None, control_leg, control,
                                           originals[leg],
                                           commands, env, str(Path(control_leg['jdk_tools']['java']['path']).parent.parent),
                                           jdk23_home,
                                           control['cases']))
        for profile in ('default', 'none'):
            jadx_label = f'jadx-{profile}-{leg}'
            cases.append(execute_candidate_case(jadx_label, 'jadx', leg,
                                               input_class, ['NestedIntUpdates'],
                                               None, jadx_path, profile, control_leg, control,
                                               originals[leg],
                                               commands, jadx_env,
                                               str(Path(control_leg['jdk_tools']['java']['path']).parent.parent),
                                               jdk23_home,
                                               control['cases']))

    # Each of the ten original behavioral checks is preserved as a named line comparison;
    # no baseline is reduced to a single opaque pass/match boolean.
    status = 'complete_baseline_recorded' if all(
        case['decompile_exit'] in (None, 0) and case['compile_exit'] is not None for case in cases) else 'baseline_recorded_with_failures'
    manifest = {
        'schema': 'nested-int-array-compound-baseline-replay-v1',
        'status': status,
        'fresh_baseline_execution': True,
        'output_root': str(OUT),
        'controls_input': {'manifest_path': str(CONTROLS_MANIFEST), 'manifest_sha256': control_hash,
                           'runner_path': control['runner']['path'], 'runner_sha256': control['runner']['sha256'],
                           'fixture_source_hashes': expected_sources,
                           'original_classes': input_records,
                           'original_runtime_streams_are_frozen_and_reused': True},
        'candidate_cli': {'path': str(cli_path), 'sha256': EXPECTED_CLI_SHA,
                          'metadata_path': str(CLI_METADATA_PATH),
                          'metadata_sha256': sha_bytes(cli_metadata_raw),
                          'product_source_snapshot': cli_metadata['candidate_sources'],
                          'provenance': 'frozen BigDecimal CLI; no product edits during this baseline run'},
        'jadx': {'manifest_path': str(JADX_BASELINE_PATH), 'manifest_sha256': sha_bytes(jadx_manifest_raw),
                 'schema': EXPECTED_JADX_SCHEMA, 'version': '1.5.6', 'version_command': version_record,
                 'launcher': jadx_record, 'distribution_files': jadx_manifest['jadx_distribution_files'],
                 'source_checkout_reference_is_not_execution_identity': jadx_manifest['source_checkout_reference'],
                 'java_home_fixed_to_javac23': jdk23_home,
                 'profiles': ['default', 'none'],
                 'flags': ['--no-res', '--config', 'none', '--threads-count', '1',
                           'default omits --rename-flags; none passes --rename-flags none']},
        'environment_policy': {'removed_for_every_process': list(ENV_KEYS_REMOVED),
                               'inherited_at_start': inherited,
                               'jadx_java_home': jdk23_home,
                               'compile_and_runtime_java_home_per_leg': True},
        'source_and_compile_policy': {
            'jarde_reports': 'render the single frozen NestedIntUpdates.class with class-source --policy single-class, JSON/evidence all; preserve its complete source text',
            'jarde_compile': 'compile generated NestedIntUpdates.java plus the exact frozen Runner.java external harness',
            'jadx_compile': 'compile every generated Java source plus the frozen Runner.java oracle; add only the target source package declaration to Runner when required',
            'classpath_sourcepath': 'fresh empty directories for every compile',
            'runtime': 'only freshly compiled classes directory on -Xverify:all classpath',
        },
        'original_cases': control['cases'],
        'input_classfiles': input_records,
        'commands': commands,
        'cases': cases,
        'file_inventory': [],
        'file_inventory_scope': 'all output files except manifest.json itself',
    }
    manifest['file_inventory'] = [file_record(path) for path in sorted(OUT.rglob('*'))
                                  if path.is_file() and path.name != 'manifest.json']
    (OUT / 'manifest.json').write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps({'status': manifest['status'], 'cases': len(cases), 'commands': len(commands),
                      'output_root': str(OUT)}, ensure_ascii=False))


if __name__ == '__main__':
    main()
