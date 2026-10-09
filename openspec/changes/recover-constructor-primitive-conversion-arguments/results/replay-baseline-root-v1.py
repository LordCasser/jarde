#!/usr/bin/env python3
"""Replay the complete draft source set against original, frozen Jarde CLI2, and JADX."""
from __future__ import annotations

import hashlib
import json
import os
import pathlib
import re
import shutil
import subprocess
import zipfile

REPO = pathlib.Path('/Users/lordcasser/workspace/projects/jarde')
RESULTS = pathlib.Path(__file__).resolve().parent
SOURCE_ROOT = RESULTS / 'draft-sources-v1'
ORIGINAL_FIXTURE = REPO / 'openspec/changes/compose-constructed-reference-array-elements/results/fixture-v1/run-003'
ORIGINAL_MANIFEST = ORIGINAL_FIXTURE / 'manifest.json'
OUT = RESULTS / 'baseline-v1'
CLI = pathlib.Path('/private/tmp/jarde-em18-composition-cli-v2')
CLI_METADATA = REPO / 'openspec/changes/compose-constructed-reference-array-elements/results/candidate-cli-v2.json'
JADX = pathlib.Path('/Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx')
SOURCE_NAMES = ('ConstructorPrimitiveConversionControls.java', 'PrimitiveLongPair.java')
CONTROL_CLASS = 'ConstructorPrimitiveConversionControls'
SOURCE_SET = [{'name': name, 'path': SOURCE_ROOT / name} for name in SOURCE_NAMES]
sha_bytes = lambda data: hashlib.sha256(data).hexdigest()
sha_file = lambda path: sha_bytes(path.read_bytes())


def save_manifest(meta: dict) -> None:
    if OUT.exists():
        meta['files'] = [
            {'path': str(path.relative_to(OUT)), 'bytes': path.stat().st_size,
             'sha256': sha_file(path)}
            for path in sorted(OUT.rglob('*'))
            if path.is_file() and path != OUT / 'manifest.json'
        ]
    (OUT / 'manifest.json').write_text(json.dumps(meta, indent=2, ensure_ascii=False) + '\n',
                                       encoding='utf-8')


def run(label: str, argv: list[pathlib.Path | str], cwd: pathlib.Path,
        commands: list[dict], env: dict[str, str] | None = None) -> tuple[subprocess.CompletedProcess[bytes], dict]:
    actual_argv = [str(value) for value in argv]
    result = subprocess.run(actual_argv, cwd=cwd, env=env, stdout=subprocess.PIPE,
                             stderr=subprocess.PIPE)
    safe_label = re.sub(r'[^A-Za-z0-9_.-]+', '_', label)
    stdout_path = OUT / 'logs' / f'{safe_label}.stdout'
    stderr_path = OUT / 'logs' / f'{safe_label}.stderr'
    stdout_path.write_bytes(result.stdout)
    stderr_path.write_bytes(result.stderr)
    record = {
        'label': label,
        'argv': actual_argv,
        'cwd': str(cwd),
        'exit': result.returncode,
        'stdout': str(stdout_path.relative_to(OUT)),
        'stdout_bytes': len(result.stdout),
        'stdout_sha256': sha_file(stdout_path),
        'stderr': str(stderr_path.relative_to(OUT)),
        'stderr_bytes': len(result.stderr),
        'stderr_sha256': sha_file(stderr_path),
    }
    if env is not None:
        record['explicit_environment'] = {key: env[key] for key in ('JAVA_HOME',) if key in env}
    commands.append(record)
    return result, record


def source_records(paths: list[pathlib.Path], out: pathlib.Path) -> list[dict]:
    records = []
    for path in paths:
        records.append({
            'path': str(path.relative_to(out)),
            'bytes': path.stat().st_size,
            'sha256': sha_file(path),
        })
    return records


def main_class_from_source(path: pathlib.Path, expected_simple_name: str | None = None) -> str:
    text = path.read_text(encoding='utf-8')
    package = re.search(r'^\s*package\s+([\w.]+)\s*;', text, re.M)
    declarations = re.findall(
        r'^\s*(?:(?:public|protected|private|abstract|final|strictfp)\s+)*class\s+([A-Za-z_$][\w$]*)\b',
        text,
        re.M,
    )
    if len(declarations) != 1:
        raise ValueError(f'{path} must declare exactly one top-level class; found {declarations}')
    simple = declarations[0]
    if expected_simple_name is not None and simple != expected_simple_name:
        raise ValueError(f'{path} declares {simple}, expected {expected_simple_name}')
    return (package.group(1) + '.' if package else '') + simple


def source_quality(paths: list[pathlib.Path], expected_stems: set[str] | None,
                   reports: list[dict]) -> tuple[bool, dict]:
    generated_stems = {path.stem for path in paths}
    no_refusal_markers = all(
        not report.get('bytecode_marker', False) and not report.get('refused_body_marker', False)
        for report in reports
    ) and all(
        '@bytecode' not in path.read_text(encoding='utf-8')
        and 'jarde_refused_body' not in path.read_text(encoding='utf-8')
        for path in paths
    )
    no_decompiler_stubs = all(
        'UnsupportedOperationException("Method not decompiled' not in path.read_text(encoding='utf-8')
        for path in paths
    )
    complete_source_set = (
        generated_stems == expected_stems
        if expected_stems is not None
        else len(paths) == 2
    )
    detail = {
        'complete_expected_source_set': complete_source_set,
        'generated_source_stems': sorted(generated_stems),
        'expected_source_stems': sorted(expected_stems) if expected_stems is not None else None,
        'all_reports_without_bytecode_or_refused_body_markers': no_refusal_markers,
        'all_sources_without_decompiler_stubs': no_decompiler_stubs,
    }
    return complete_source_set and no_refusal_markers and no_decompiler_stubs, detail


def create_input_jar(classes: pathlib.Path, class_paths: list[pathlib.Path], destination: pathlib.Path) -> dict:
    class_names = sorted(path.name for path in class_paths)
    if class_names != sorted(f'{pathlib.Path(name).stem}.class' for name in SOURCE_NAMES):
        raise ValueError(f'baseline input must contain exactly the two draft classes: {class_names}')
    with zipfile.ZipFile(destination, 'w') as jar:
        for path in sorted(class_paths):
            jar.write(path, path.name)
    return {
        'path': str(destination.relative_to(OUT)),
        'sha256': sha_file(destination),
        'class_entries': [
            {'name': path.name, 'bytes': path.stat().st_size, 'sha256': sha_file(path)}
            for path in sorted(class_paths)
        ],
    }


def main() -> int:
    cli_metadata = json.loads(CLI_METADATA.read_text(encoding='utf-8'))
    cli_metadata_sha = sha_file(CLI_METADATA)
    cli_sha = sha_file(CLI)
    if pathlib.Path(cli_metadata['cli_path']) != CLI:
        raise SystemExit('CLI path differs from the frozen candidate-cli-v2 metadata')
    if cli_metadata['cli_sha256'] != cli_sha:
        raise SystemExit('CLI binary SHA-256 differs from the frozen candidate-cli-v2 metadata')
    OUT.mkdir()  # Existing baseline evidence is never overwritten.
    (OUT / 'logs').mkdir()
    original_manifest = json.loads(ORIGINAL_MANIFEST.read_text(encoding='utf-8'))
    original_manifest_sha = sha_file(ORIGINAL_MANIFEST)
    jadx_sha = sha_file(JADX)
    original_sources = [item['path'] for item in SOURCE_SET]
    expected_stems = {pathlib.Path(name).stem for name in SOURCE_NAMES}
    for path in original_sources:
        if not path.is_file():
            raise SystemExit(f'missing frozen draft source: {path}')
    if {path.name for path in original_sources} != set(SOURCE_NAMES):
        raise SystemExit('draft source set differs from the two-class frozen closure')

    meta = {
        'schema': 'recover-constructor-primitive-conversion-baseline-root-v1',
        'runner_path': str(pathlib.Path(__file__).resolve()),
        'runner_sha256': sha_file(pathlib.Path(__file__).resolve()),
        'original_fixture_manifest': str(ORIGINAL_MANIFEST),
        'original_fixture_manifest_sha256': original_manifest_sha,
        'cli_path': str(CLI),
        'cli_sha256': cli_sha,
        'cli_metadata_path': str(CLI_METADATA),
        'cli_metadata_sha256': cli_metadata_sha,
        'cli_metadata_cli_path': cli_metadata['cli_path'],
        'cli_metadata_cli_sha256': cli_metadata['cli_sha256'],
        'jadx_path': str(JADX),
        'jadx_sha256': jadx_sha,
        'source_closure': [
            {'path': str(path.relative_to(REPO)), 'bytes': path.stat().st_size,
             'sha256': sha_file(path)} for path in original_sources
        ],
        'jdk_legs': {},
        'commands': [],
        'cases': [],
        'scope_note': 'original source and class-set inputs stay closed at two classes; each generated source set compiles and runs from its own isolated class directory; no stream normalization',
    }

    def save() -> None:
        save_manifest(meta)

    for leg, jdk in original_manifest['jdk_legs'].items():
        home = pathlib.Path(jdk['jdk_home'])
        compiler_flags = list(jdk['compiler_flags'])
        javac = home / 'bin/javac'
        java = home / 'bin/java'
        original_work = OUT / 'original' / leg
        source_dir = original_work / 'sources'
        original_classes = original_work / 'classes'
        empty = original_work / 'empty-classpath-sourcepath'
        for directory in (source_dir, original_classes, empty):
            directory.mkdir(parents=True)
        copied_sources = []
        for source in original_sources:
            copy = source_dir / source.name
            shutil.copyfile(source, copy)
            copied_sources.append(copy)
        original_compile, original_compile_record = run(
            f'{leg}-original-compile',
            [javac, *compiler_flags, '-g:none', '-classpath', empty, '-sourcepath', empty,
             '-d', original_classes, *sorted(copied_sources)],
            original_work,
            meta['commands'],
        )
        original_class_files = sorted(original_classes.rglob('*.class'))
        original_class_set = [str(path.relative_to(original_classes)) for path in original_class_files]
        original_compile_ok = (
            original_compile.returncode == 0
            and {path.name for path in original_class_files}
            == {pathlib.Path(name).stem + '.class' for name in SOURCE_NAMES}
        )
        original_runtime = None
        input_jar_record = None
        if original_compile_ok:
            original_run, original_runtime = run(
                f'{leg}-original-run',
                [java, '-Xverify:all', '-cp', original_classes, CONTROL_CLASS],
                original_work,
                meta['commands'],
            )
            input_jar = OUT / 'inputs' / f'{leg}-original-two-class.jar'
            input_jar.parent.mkdir(exist_ok=True)
            input_jar_record = create_input_jar(original_classes, original_class_files, input_jar)
            original_runtime_ok = original_run.returncode == 0
        else:
            original_runtime_ok = False

        original_source_set = source_records(copied_sources, OUT)
        original_class_records = source_records(original_class_files, OUT)
        meta['jdk_legs'][leg] = {
            'jdk_home': str(home),
            'compiler_flags_from_fixture_v1_run003': compiler_flags,
            'javac_sha256': sha_file(javac),
            'java_sha256': sha_file(java),
            'empty_classpath_sourcepath': str(empty),
            'original_sources': original_source_set,
            'original_compile': original_compile_record,
            'original_compile_ok': original_compile_ok,
            'original_class_set': original_class_set,
            'original_classes': original_class_records,
            'original_input_jar': input_jar_record,
            'original_runtime': original_runtime,
            'original_runtime_ok': original_runtime_ok,
        }
        save()

        if input_jar_record is None:
            meta['cases'].append({
                'leg': leg, 'profile': 'jarde-cli2',
                'original_compile_ok': original_compile_ok,
                'original_runtime_ok': original_runtime_ok,
                'stage_render_ok': False, 'compile_ok': False, 'quality_ok': False,
                'candidate_runtime_exit0': False,
                'rawstreams_match': False, 'exit_match': False, 'accepted': False,
                'stop_reason': 'original source set did not compile into the required two-class closure',
            })
            save()
            continue

        for profile in ('jarde-cli2', 'jadx-none', 'jadx-default'):
            work = OUT / 'generated' / leg / profile
            sources = work / 'sources'
            classes = work / 'classes'
            generated_empty = work / 'empty-classpath-sourcepath'
            sources.mkdir(parents=True)
            classes.mkdir()
            generated_empty.mkdir()
            reports: list[dict] = []
            main_source: pathlib.Path | None = None

            if profile == 'jarde-cli2':
                render_source_dir = sources
                for class_name in sorted(pathlib.Path(name).stem for name in SOURCE_NAMES):
                    rendered, rendered_record = run(
                        f'{leg}-{profile}-render-{class_name}',
                        [CLI, 'class-source', '--input', OUT / input_jar_record['path'],
                         '--class', class_name, '--policy', 'plain-jar', '--format', 'json',
                         '--evidence', 'all', '--release', '8'],
                        REPO,
                        meta['commands'],
                    )
                    report = {
                        'class': class_name,
                        'render_command': rendered_record,
                        'json_parsed': False,
                        'bytecode_marker': None,
                        'refused_body_marker': None,
                        'source_path': None,
                        'source_sha256': None,
                    }
                    if rendered.returncode == 0:
                        try:
                            document = json.loads(rendered.stdout)
                            report_text = document['text']
                            source = render_source_dir / f'{class_name}.java'
                            source.write_text(report_text, encoding='utf-8')
                            report.update({
                                'json_parsed': True,
                                'bytecode_marker': '@bytecode' in report_text,
                                'refused_body_marker': 'jarde_refused_body' in report_text,
                                'source_path': str(source.relative_to(OUT)),
                                'source_sha256': sha_file(source),
                            })
                            if class_name == CONTROL_CLASS:
                                main_source = source
                        except (json.JSONDecodeError, KeyError, TypeError) as error:
                            report['json_parse_error'] = str(error)
                        reports.append(report)
                    else:
                        reports.append(report)
                    save()
                generated_paths = sorted(sources.rglob('*.java'))
            else:
                env = dict(os.environ)
                env['JAVA_HOME'] = str(original_manifest['jdk_legs']['javac23']['jdk_home'])
                jadx_output = work / 'jadx-output'
                argv = [JADX, '--no-res', '-d', jadx_output]
                if profile == 'jadx-none':
                    argv += ['--rename-flags', 'none']
                decompile, decompile_record = run(
                    f'{leg}-{profile}-decompile',
                    [*argv, OUT / input_jar_record['path']],
                    REPO,
                    meta['commands'],
                    env,
                )
                sources = jadx_output / 'sources'
                generated_paths = sorted(sources.rglob('*.java')) if sources.exists() else []
                main_candidates = [
                    path for path in generated_paths
                    if re.search(
                        r'\bstatic\s+void\s+main\s*\(\s*(?:java\.lang\.)?String\s*(?:\[\s*\]|\.\.\.)\s+[A-Za-z_$][\w$]*\s*\)',
                        path.read_text(encoding='utf-8'),
                    )
                ]
                if len(main_candidates) == 1:
                    main_source = main_candidates[0]
                reports.append({
                    'decompile_command': decompile_record,
                    'decompile_exit': decompile.returncode,
                    'generated_source_set': source_records(generated_paths, OUT),
                    'unsupported_decompiler_stubs': [
                        str(path.relative_to(OUT)) for path in generated_paths
                        if 'UnsupportedOperationException("Method not decompiled' in path.read_text(encoding='utf-8')
                    ],
                })
                save()

            stage_render_ok = (
                len(reports) == len(SOURCE_NAMES)
                and all(
                    report['render_command']['exit'] == 0 and report['json_parsed']
                    for report in reports
                )
                if profile == 'jarde-cli2'
                else len(reports) == 1 and reports[0]['decompile_exit'] == 0
            )
            quality_ok, quality_detail = source_quality(
                generated_paths,
                expected_stems if profile == 'jarde-cli2' else None,
                reports,
            )
            quality_detail['stage_render_ok'] = stage_render_ok
            main_name = None
            main_parse_error = None
            if main_source is not None:
                try:
                    main_name = main_class_from_source(
                        main_source,
                        CONTROL_CLASS if profile == 'jarde-cli2' else None,
                    )
                except (OSError, ValueError) as error:
                    main_parse_error = str(error)
            else:
                main_parse_error = f'no unique {CONTROL_CLASS}.java source was generated'
            quality_ok = quality_ok and stage_render_ok and main_name is not None
            compile_record = None
            compile_ok = False
            runtime_record = None
            runtime = None
            if generated_paths:
                compile_result, compile_record = run(
                    f'{leg}-{profile}-compile',
                    [javac, *compiler_flags, '-g:none', '-classpath', generated_empty,
                     '-sourcepath', generated_empty, '-d', classes, *generated_paths],
                    work,
                    meta['commands'],
                )
                compile_ok = compile_result.returncode == 0
                if compile_ok and main_name is not None:
                    _, runtime_record = run(
                        f'{leg}-{profile}-run',
                        [java, '-Xverify:all', '-cp', classes, main_name],
                        work,
                        meta['commands'],
                    )
                    runtime = runtime_record
            original_runtime_hashes = original_runtime or {}
            rawstreams_match = bool(
                runtime is not None
                and runtime['stdout_sha256'] == original_runtime_hashes.get('stdout_sha256')
                and runtime['stderr_sha256'] == original_runtime_hashes.get('stderr_sha256')
            )
            exit_match = bool(
                runtime is not None
                and original_runtime_hashes.get('exit') == runtime.get('exit')
            )
            original_leg = meta['jdk_legs'][leg]
            candidate_runtime_exit0 = runtime is not None and runtime['exit'] == 0
            case = {
                'leg': leg,
                'profile': profile,
                'input_jar': input_jar_record,
                'input_class_set': input_jar_record['class_entries'],
                'generated_source_set': source_records(generated_paths, OUT),
                'reports': reports,
                'quality': quality_detail,
                'quality_ok': quality_ok,
                'stage_render_ok': stage_render_ok,
                'compile': compile_record,
                'compile_ok': compile_ok,
                'original_compile_ok': original_leg['original_compile_ok'],
                'original_runtime_ok': original_leg['original_runtime_ok'],
                'resolved_main_class': main_name,
                'main_class_parse_error': main_parse_error,
                'runtime': runtime,
                'candidate_runtime_exit0': candidate_runtime_exit0,
                'rawstreams_match': rawstreams_match,
                'exit_match': exit_match,
                'accepted': (
                    original_leg['original_compile_ok']
                    and original_leg['original_runtime_ok']
                    and stage_render_ok
                    and compile_ok
                    and quality_ok
                    and candidate_runtime_exit0
                    and rawstreams_match
                    and exit_match
                ),
            }
            meta['cases'].append(case)
            save()

    save()
    print(json.dumps({
        'manifest': str((OUT / 'manifest.json').relative_to(REPO)),
        'runner_sha256': meta['runner_sha256'],
        'cli_sha256': meta['cli_sha256'],
        'source_closure': meta['source_closure'],
        'legs': list(meta['jdk_legs']),
        'profiles': ['jarde-cli2', 'jadx-none', 'jadx-default'],
        'accepted_cases': sum(bool(case.get('accepted')) for case in meta['cases']),
        'cases': len(meta['cases']),
    }, indent=2, ensure_ascii=False))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
