#!/usr/bin/env python3
"""Independently verify the saved baseline-v1 manifest and raw evidence."""
from __future__ import annotations

import hashlib
import json
import pathlib
import re
import zipfile

REPO = pathlib.Path('/Users/lordcasser/workspace/projects/jarde')
RESULTS = pathlib.Path(__file__).resolve().parent
BASELINE = RESULTS / 'baseline-v1'
MANIFEST = BASELINE / 'manifest.json'
OUTPUT = RESULTS / 'baseline-root-verification-v1.json'
CLI_METADATA = REPO / 'openspec/changes/compose-constructed-reference-array-elements/results/candidate-cli-v2.json'
ORIGINAL_FIXTURE_MANIFEST = (
    REPO / 'openspec/changes/compose-constructed-reference-array-elements/results/fixture-v1/run-003/manifest.json'
)


def sha_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha_file(path: pathlib.Path) -> str:
    return sha_bytes(path.read_bytes())


def main() -> int:
    if OUTPUT.exists():
        raise SystemExit(f'refusing to overwrite verifier output: {OUTPUT}')
    manifest = json.loads(MANIFEST.read_text(encoding='utf-8'))
    original_fixture_manifest = json.loads(ORIGINAL_FIXTURE_MANIFEST.read_text(encoding='utf-8'))
    errors: list[str] = []

    def require(ok: bool, message: str) -> None:
        if not ok:
            errors.append(message)

    def evidence_path(relative: str) -> pathlib.Path:
        return BASELINE / relative

    def empty_directory(path: pathlib.Path) -> bool:
        return path.is_dir() and not any(path.iterdir())

    def ordered_subsequence(argv: list[str], values: list[str]) -> bool:
        return argv[1:1 + len(values)] == values

    def source_record_map(records: list[dict]) -> dict[str, dict]:
        return {pathlib.Path(item['path']).name: item for item in records}

    def declared_class_name(path: pathlib.Path) -> str | None:
        text = path.read_text(encoding='utf-8')
        package = re.search(r'^\s*package\s+([\w.]+)\s*;', text, re.M)
        declarations = re.findall(
            r'^\s*(?:(?:public|protected|private|abstract|final|strictfp)\s+)*class\s+([A-Za-z_$][\w$]*)\b',
            text, re.M,
        )
        if len(declarations) != 1:
            return None
        return (package.group(1) + '.' if package else '') + declarations[0]

    main_pattern = re.compile(
        r'\bstatic\s+void\s+main\s*\(\s*(?:java\.lang\.)?String\s*(?:\[\s*\]|\.\.\.)\s+[A-Za-z_$][\w$]*\s*\)'
    )

    # Check the closed inventory and every actual command stream independently.
    listed_files: dict[str, dict] = {}
    for item in manifest.get('files', []):
        relative = item['path']
        listed_files[relative] = item
        path = evidence_path(relative)
        require(path.is_file(), f'manifest evidence file missing: {relative}')
        if path.is_file():
            require(path.stat().st_size == item['bytes'], f'evidence byte count differs: {relative}')
            require(sha_file(path) == item['sha256'], f'evidence SHA-256 differs: {relative}')
    actual_files = {
        str(path.relative_to(BASELINE)) for path in BASELINE.rglob('*')
        if path.is_file() and path != MANIFEST
    }
    require(actual_files == set(listed_files), 'manifest file inventory does not match baseline-v1 contents')

    command_by_label: dict[str, dict] = {}
    for command in manifest.get('commands', []):
        label = command['label']
        require(label not in command_by_label, f'duplicate command label: {label}')
        command_by_label[label] = command
        for stream in ('stdout', 'stderr'):
            path = evidence_path(command[stream])
            require(path.is_file(), f'{label}: missing {stream} capture')
            if path.is_file():
                raw = path.read_bytes()
                require(len(raw) == command[f'{stream}_bytes'], f'{label}: {stream} byte count differs')
                require(sha_bytes(raw) == command[f'{stream}_sha256'], f'{label}: {stream} SHA-256 differs')
        require(bool(command.get('argv')) and pathlib.Path(command['argv'][0]).is_absolute(),
                f'{label}: argv lacks an absolute executable path')

    def command(label: str | None) -> dict | None:
        if label is None:
            return None
        found = command_by_label.get(label)
        require(found is not None, f'missing recorded command: {label}')
        return found

    def check_recorded_command(record: dict | None, label: str, expected_exit: int | None = None) -> dict | None:
        actual = command(label)
        require(record is not None, f'missing manifest command record: {label}')
        if actual is not None and record is not None:
            require(record == actual, f'{label}: embedded command record differs from command ledger')
            if expected_exit is not None:
                require(actual['exit'] == expected_exit, f'{label}: expected exit {expected_exit}, got {actual["exit"]}')
        return actual

    # Tie the requested CLI2 identity to its separately frozen metadata.
    require(CLI_METADATA.is_file(), 'frozen CLI2 metadata is missing')
    if CLI_METADATA.is_file():
        metadata = json.loads(CLI_METADATA.read_text(encoding='utf-8'))
        metadata_sha = sha_file(CLI_METADATA)
        cli_path = pathlib.Path(manifest['cli_path'])
        require(manifest.get('cli_metadata_path') == str(CLI_METADATA), 'manifest CLI metadata path differs')
        require(manifest.get('cli_metadata_sha256') == metadata_sha, 'manifest CLI metadata SHA-256 differs')
        require(metadata.get('cli_path') == str(cli_path), 'CLI binary path differs from frozen metadata')
        require(manifest.get('cli_metadata_cli_path') == metadata.get('cli_path'), 'recorded metadata CLI path differs')
        require(manifest.get('cli_metadata_cli_sha256') == metadata.get('cli_sha256'), 'recorded metadata CLI SHA differs')
        require(cli_path.is_file(), 'CLI2 executable is missing')
        if cli_path.is_file():
            require(sha_file(cli_path) == metadata.get('cli_sha256'), 'actual CLI2 executable differs from frozen metadata')
            require(manifest.get('cli_sha256') == sha_file(cli_path), 'manifest CLI2 SHA differs from actual executable')

    require(len(manifest.get('jdk_legs', {})) == 2, 'expected exactly two frozen JDK legs')
    require(set(manifest.get('jdk_legs', {})) == {'javac8', 'javac23'}, 'unexpected frozen JDK leg names')
    require(len(manifest.get('cases', [])) == 6, 'expected all six profile/JDK comparison legs')
    require(manifest.get('profiles') in (None, ['jarde-cli2', 'jadx-none', 'jadx-default']),
            'unexpected profile declaration')

    require(pathlib.Path(manifest['original_fixture_manifest']) == ORIGINAL_FIXTURE_MANIFEST,
            'original fixture manifest path differs')
    require(ORIGINAL_FIXTURE_MANIFEST.is_file()
            and sha_file(ORIGINAL_FIXTURE_MANIFEST) == manifest.get('original_fixture_manifest_sha256'),
            'original fixture manifest hash differs')
    require(manifest.get('original_fixture_manifest_sha256') == sha_file(ORIGINAL_FIXTURE_MANIFEST),
            'original fixture manifest recorded SHA-256 differs')
    runner_path = pathlib.Path(manifest['runner_path'])
    require(runner_path.is_file() and sha_file(runner_path) == manifest.get('runner_sha256'),
            'baseline runner path/hash differs')
    jadx_path = pathlib.Path(manifest['jadx_path'])
    require(jadx_path.is_file() and sha_file(jadx_path) == manifest.get('jadx_sha256'),
            'JADX launcher path/hash differs')

    draft_sources = {pathlib.Path(item['path']).name: item for item in manifest.get('source_closure', [])}
    require(set(draft_sources) == {'ConstructorPrimitiveConversionControls.java', 'PrimitiveLongPair.java'},
            'frozen draft source closure differs from the two-class input')
    for name, item in draft_sources.items():
        source = REPO / item['path']
        require(source.is_file(), f'frozen draft source is missing: {name}')
        if source.is_file():
            require(source.stat().st_size == item['bytes'] and sha_file(source) == item['sha256'],
                    f'frozen draft source hash differs: {name}')

    for leg, leg_data in manifest.get('jdk_legs', {}).items():
        home = pathlib.Path(leg_data['jdk_home'])
        javac = home / 'bin/javac'
        java = home / 'bin/java'
        fixture_leg = original_fixture_manifest.get('jdk_legs', {}).get(leg, {})
        fixture_flags = fixture_leg.get('compiler_flags', [])
        require(home == pathlib.Path(fixture_leg.get('jdk_home', '')), f'{leg}: JDK home differs from run-003')
        require(leg_data.get('compiler_flags_from_fixture_v1_run003') == fixture_flags,
                f'{leg}: recorded compiler flags differ from run-003')
        require(javac.is_file() and sha_file(javac) == leg_data['javac_sha256'], f'{leg}: javac identity differs')
        require(java.is_file() and sha_file(java) == leg_data['java_sha256'], f'{leg}: java identity differs')
        original_sources = leg_data.get('original_sources', [])
        original_classes = leg_data.get('original_classes', [])
        require(len(original_sources) == 2, f'{leg}: original source set is not two classes')
        require(len(original_classes) == 2, f'{leg}: original class set is not two classes')
        original_class_names = {pathlib.Path(item['path']).name for item in original_classes}
        require(original_class_names == {'ConstructorPrimitiveConversionControls.class', 'PrimitiveLongPair.class'},
                f'{leg}: original class closure differs')
        actual_original_sources = {
            path for path in (BASELINE / 'original' / leg / 'sources').rglob('*.java')
            if path.is_file()
        }
        recorded_original_sources = {evidence_path(item['path']) for item in original_sources}
        require(len(recorded_original_sources) == len(original_sources),
                f'{leg}: duplicate original source records')
        require(actual_original_sources == recorded_original_sources,
                f'{leg}: actual copied source directory differs from manifest source closure')
        copied_by_name = source_record_map(original_sources)
        for name, frozen in draft_sources.items():
            copied_record = copied_by_name.get(name)
            require(copied_record is not None, f'{leg}: copied draft source missing from record: {name}')
            draft = REPO / frozen['path']
            copied = evidence_path(copied_record['path']) if copied_record else None
            if copied and copied.is_file() and draft.is_file():
                require(draft.read_bytes() == copied.read_bytes(), f'{leg}: copied draft source bytes differ: {name}')
                require(copied_record['sha256'] == frozen['sha256'] and copied_record['bytes'] == frozen['bytes'],
                        f'{leg}: copied draft source record differs from frozen source: {name}')
        class_dir = BASELINE / 'original' / leg / 'classes'
        actual_class_paths = {path for path in class_dir.rglob('*.class') if path.is_file()}
        recorded_class_paths = {evidence_path(item['path']) for item in original_classes}
        require(actual_class_paths == recorded_class_paths, f'{leg}: actual original class directory differs from manifest')
        require({str(path.relative_to(class_dir)) for path in actual_class_paths}
                == set(leg_data.get('original_class_set', [])),
                f'{leg}: actual original class set differs from closed class inventory')
        require(leg_data.get('original_class_set') == ['ConstructorPrimitiveConversionControls.class', 'PrimitiveLongPair.class'],
                f'{leg}: recorded original class set is not the exact two-class closure')

        compile_cmd = check_recorded_command(leg_data.get('original_compile'), f'{leg}-original-compile', 0)
        require(leg_data.get('original_compile_ok') is True, f'{leg}: original compile flag is false')
        if compile_cmd:
            argv = compile_cmd['argv']
            require(pathlib.Path(argv[0]) == javac, f'{leg}: original compile did not use the frozen javac')
            require(ordered_subsequence(argv, fixture_flags), f'{leg}: original javac flags/order differ from run-003')
            require('-g:none' in argv and '-classpath' in argv and '-sourcepath' in argv,
                    f'{leg}: original compile lacks debug/isolated path flags')
            require('-d' in argv and argv[argv.index('-d') + 1] == str(BASELINE / 'original' / leg / 'classes'),
                    f'{leg}: original compile output directory differs')
            for flag in ('-classpath', '-sourcepath'):
                idx = argv.index(flag) if flag in argv else -1
                require(idx >= 0 and idx + 1 < len(argv) and pathlib.Path(argv[idx + 1]).is_dir(),
                        f'{leg}: {flag} directory is missing')
                if idx >= 0 and idx + 1 < len(argv):
                    require(empty_directory(pathlib.Path(argv[idx + 1])),
                            f'{leg}: {flag} directory is not actually empty')
                    require(argv[idx + 1] == argv[argv.index('-classpath') + 1],
                            f'{leg}: classpath/sourcepath are not the same explicit empty directory')
            passed_sources = {pathlib.Path(arg).resolve() for arg in argv if arg.endswith('.java')}
            recorded_sources = {(BASELINE / item['path']).resolve() for item in original_sources}
            require(passed_sources == recorded_sources, f'{leg}: original compile source arguments differ from complete source set')
            require(compile_cmd['cwd'] == str(BASELINE / 'original' / leg), f'{leg}: original compile cwd differs')

        jar_record = leg_data.get('original_input_jar')
        require(jar_record is not None, f'{leg}: original input JAR missing')
        if jar_record:
            jar_path = evidence_path(jar_record['path'])
            require(jar_path.is_file() and sha_file(jar_path) == jar_record['sha256'], f'{leg}: input JAR hash differs')
            if jar_path.is_file():
                with zipfile.ZipFile(jar_path) as jar:
                    file_entries = [info for info in jar.infolist() if not info.is_dir()]
                    entries = {info.filename: jar.read(info.filename) for info in file_entries}
                require(len(file_entries) == len(entries), f'{leg}: duplicate JAR entry names')
                expected_entries = {item['name']: item for item in jar_record['class_entries']}
                require(set(entries) == set(expected_entries), f'{leg}: input JAR entries differ from complete class set')
                for name, content in entries.items():
                    require(len(content) == expected_entries[name]['bytes'], f'{leg}: JAR entry byte count differs: {name}')
                    require(sha_bytes(content) == expected_entries[name]['sha256'], f'{leg}: JAR entry hash differs: {name}')
                    class_file = next((item for item in original_classes if pathlib.Path(item['path']).name == name), None)
                    require(class_file is not None, f'{leg}: JAR entry has no original class record: {name}')
                    if class_file:
                        raw_class = evidence_path(class_file['path']).read_bytes()
                        require(raw_class == content, f'{leg}: JAR entry bytes differ from compiled original class: {name}')

        original_runtime = leg_data.get('original_runtime')
        original_runtime_cmd = check_recorded_command(original_runtime, f'{leg}-original-run', 0)
        require(leg_data.get('original_runtime_ok') is True, f'{leg}: original runtime flag is false')
        if original_runtime_cmd:
            argv = original_runtime_cmd['argv']
            expected_classes = str(BASELINE / 'original' / leg / 'classes')
            require(pathlib.Path(argv[0]) == java, f'{leg}: original runtime did not use frozen java')
            require('-Xverify:all' in argv, f'{leg}: original runtime lacks -Xverify:all')
            require('-cp' in argv and argv[argv.index('-cp') + 1] == expected_classes,
                    f'{leg}: original runtime classpath is not its original class directory')
            require(argv[-1] == 'ConstructorPrimitiveConversionControls', f'{leg}: original runtime main differs')

    case_by_key = {(case.get('leg'), case.get('profile')): case for case in manifest.get('cases', [])}
    require(len(case_by_key) == 6, 'comparison cases do not uniquely cover the six legs')
    comparisons = []
    for leg in ('javac8', 'javac23'):
        leg_data = manifest['jdk_legs'][leg]
        java = pathlib.Path(leg_data['jdk_home']) / 'bin/java'
        javac = pathlib.Path(leg_data['jdk_home']) / 'bin/javac'
        original_runtime = leg_data['original_runtime']
        original_stdout = evidence_path(original_runtime['stdout']).read_bytes()
        original_stderr = evidence_path(original_runtime['stderr']).read_bytes()
        for profile in ('jarde-cli2', 'jadx-none', 'jadx-default'):
            case = case_by_key.get((leg, profile))
            require(case is not None, f'missing comparison case {leg}/{profile}')
            if case is None:
                continue
            reports = case.get('reports', [])
            stage_render_ok = False
            marker_summary = []
            generated_records = case.get('generated_source_set', [])
            generated_paths = [evidence_path(item['path']) for item in generated_records]
            require(all(path.is_file() for path in generated_paths), f'{leg}/{profile}: generated source missing')
            source_directory = (BASELINE / 'generated' / leg / profile / 'sources'
                                if profile == 'jarde-cli2'
                                else BASELINE / 'generated' / leg / profile / 'jadx-output' / 'sources')
            actual_source_paths = {path for path in source_directory.rglob('*.java') if path.is_file()}
            require(len(generated_paths) == len(set(generated_paths)),
                    f'{leg}/{profile}: duplicate generated source records')
            require(actual_source_paths == set(generated_paths),
                    f'{leg}/{profile}: generated source records are not the complete actual source-directory closure')
            for item, path in zip(generated_records, generated_paths):
                if path.is_file():
                    require(path.stat().st_size == item['bytes'] and sha_file(path) == item['sha256'],
                            f'{leg}/{profile}: generated source record differs: {item["path"]}')

            if profile == 'jarde-cli2':
                require(len(reports) == 2, f'{leg}/{profile}: expected two class-source reports')
                report_ok = len(reports) == 2
                report_classes = set()
                for report in reports:
                    report_classes.add(report.get('class'))
                    render_cmd = report.get('render_command')
                    label = render_cmd.get('label') if render_cmd else None
                    actual = check_recorded_command(render_cmd, label, 0) if label else None
                    parsed = False
                    if actual:
                        argv = actual['argv']
                        require(pathlib.Path(argv[0]) == pathlib.Path(manifest['cli_path']),
                                f'{leg}/{profile}: CLI2 render did not use frozen CLI executable')
                        jar_path = str(evidence_path(case['input_jar']['path']))
                        require(argv[1:2] == ['class-source'] and '--input' in argv
                                and argv[argv.index('--input') + 1] == jar_path,
                                f'{leg}/{profile}: CLI2 render argv does not identify the frozen input JAR')
                        require('--class' in argv and argv[argv.index('--class') + 1] == report.get('class'),
                                f'{leg}/{profile}: CLI2 render argv class differs from report')
                        expected_options = (('--policy', 'plain-jar'), ('--format', 'json'),
                                            ('--evidence', 'all'), ('--release', '8'))
                        for option, value in expected_options:
                            require(option in argv and argv[argv.index(option) + 1] == value,
                                    f'{leg}/{profile}: CLI2 render argv missing {option} {value}')
                        raw = evidence_path(actual['stdout']).read_bytes()
                        try:
                            document = json.loads(raw)
                            text = document['text']
                            parsed = True
                            source = evidence_path(report['source_path']) if report.get('source_path') else None
                            require(source is not None and source.is_file(), f'{leg}/{profile}: rendered source missing')
                            if source and source.is_file():
                                require(source.read_text(encoding='utf-8') == text,
                                        f'{leg}/{profile}: saved source differs from render JSON text')
                                require(sha_file(source) == report.get('source_sha256'),
                                        f'{leg}/{profile}: source hash field differs')
                            actual_bytecode = '@bytecode' in text
                            actual_refused = 'jarde_refused_body' in text
                            require(report.get('bytecode_marker') == actual_bytecode,
                                    f'{leg}/{profile}: bytecode marker field differs for {report.get("class")}')
                            require(report.get('refused_body_marker') == actual_refused,
                                    f'{leg}/{profile}: refusal marker field differs for {report.get("class")}')
                            marker_summary.append({'class': report.get('class'), 'bytecode': actual_bytecode,
                                                   'refused_body': actual_refused})
                        except (json.JSONDecodeError, KeyError, TypeError) as error:
                            errors.append(f'{leg}/{profile}: invalid render JSON for {report.get("class")}: {error}')
                    require(report.get('json_parsed') is parsed,
                            f'{leg}/{profile}: JSON parsed field differs for {report.get("class")}')
                    report_ok = report_ok and actual is not None and actual['exit'] == 0 and parsed
                require(report_classes == {'ConstructorPrimitiveConversionControls', 'PrimitiveLongPair'},
                        f'{leg}/{profile}: rendered class set differs')
                stage_render_ok = report_ok
            else:
                require(len(reports) == 1, f'{leg}/{profile}: expected one JADX report')
                if reports:
                    report = reports[0]
                    decompile_cmd = report.get('decompile_command')
                    label = decompile_cmd.get('label') if decompile_cmd else None
                    actual = check_recorded_command(decompile_cmd, label, 0) if label else None
                    stage_render_ok = actual is not None and actual['exit'] == 0
                    if actual:
                        argv = actual['argv']
                        jar_path = str(evidence_path(case['input_jar']['path']))
                        require(pathlib.Path(argv[0]) == pathlib.Path(manifest['jadx_path']),
                                f'{leg}/{profile}: decompile did not use frozen JADX executable')
                        require('--no-res' in argv and argv[-1] == jar_path,
                                f'{leg}/{profile}: JADX argv does not use frozen input JAR')
                        require(('--rename-flags' in argv) == (profile == 'jadx-none'),
                                f'{leg}/{profile}: JADX rename profile differs')
                        if profile == 'jadx-none':
                            require(argv[argv.index('--rename-flags') + 1] == 'none',
                                    f'{leg}/{profile}: JADX rename setting differs')
                        require('-d' in argv and argv[argv.index('-d') + 1]
                                == str(BASELINE / 'generated' / leg / profile / 'jadx-output'),
                                f'{leg}/{profile}: JADX output directory differs')
                    require(report.get('decompile_exit') == (actual['exit'] if actual else None),
                            f'{leg}/{profile}: decompile exit field differs')
                    listed_generated = {item['path']: item for item in report.get('generated_source_set', [])}
                    actual_generated = {str(path.relative_to(BASELINE)): path for path in generated_paths}
                    require(set(listed_generated) == set(actual_generated),
                            f'{leg}/{profile}: JADX report source set differs from compiled source set')
                    for relative, path in actual_generated.items():
                        if path.is_file():
                            record = listed_generated[relative]
                            require(path.stat().st_size == record['bytes'] and sha_file(path) == record['sha256'],
                                    f'{leg}/{profile}: JADX source record differs: {relative}')
                            text = path.read_text(encoding='utf-8')
                            marker_summary.append({'source': relative,
                                                   'bytecode': '@bytecode' in text,
                                                   'refused_body': 'jarde_refused_body' in text,
                                                   'unsupported_stub': 'UnsupportedOperationException("Method not decompiled' in text})
                    expected_stubs = [
                        str(path.relative_to(BASELINE)) for path in generated_paths if path.is_file()
                        and 'UnsupportedOperationException("Method not decompiled' in path.read_text(encoding='utf-8')
                    ]
                    require(report.get('unsupported_decompiler_stubs') == expected_stubs,
                            f'{leg}/{profile}: unsupported-stub list differs')

            require(case.get('stage_render_ok') is stage_render_ok,
                    f'{leg}/{profile}: stage_render_ok differs from actual render evidence')
            require(case.get('input_jar') == leg_data.get('original_input_jar'),
                    f'{leg}/{profile}: comparison input differs from the leg original JAR')
            quality = case.get('quality', {})
            require(quality.get('stage_render_ok') is stage_render_ok,
                    f'{leg}/{profile}: quality stage_render_ok differs')
            expected_stems = {path.stem for path in generated_paths}
            no_refusal = all('@bytecode' not in path.read_text(encoding='utf-8')
                             and 'jarde_refused_body' not in path.read_text(encoding='utf-8')
                             for path in generated_paths if path.is_file())
            no_stubs = all('UnsupportedOperationException("Method not decompiled' not in path.read_text(encoding='utf-8')
                           for path in generated_paths if path.is_file())
            complete_sources = expected_stems == {'ConstructorPrimitiveConversionControls', 'PrimitiveLongPair'}
            main_sources = [path for path in generated_paths if path.is_file()
                            and main_pattern.search(path.read_text(encoding='utf-8'))]
            main_names = [declared_class_name(path) for path in main_sources]
            actual_main_ok = (len(main_names) == 1 and main_names[0] is not None
                              and case.get('resolved_main_class') == main_names[0]
                              and case.get('main_class_parse_error') is None)
            if profile == 'jarde-cli2':
                expected_quality = complete_sources and no_refusal and no_stubs and stage_render_ok and actual_main_ok
            else:
                expected_quality = (len(generated_paths) == 2 and no_refusal and no_stubs
                                    and stage_render_ok and actual_main_ok)
            require(case.get('quality_ok') is expected_quality,
                    f'{leg}/{profile}: quality_ok differs from source/render facts')
            require(quality.get('all_reports_without_bytecode_or_refused_body_markers') is no_refusal,
                    f'{leg}/{profile}: source refusal-marker summary differs')
            require(quality.get('all_sources_without_decompiler_stubs') is no_stubs,
                    f'{leg}/{profile}: source stub summary differs')

            compile_record = case.get('compile')
            compile_ok = False
            compile_label = f'{leg}-{profile}-compile'
            actual_compile = check_recorded_command(compile_record, compile_label)
            if actual_compile:
                compile_ok = actual_compile['exit'] == 0
                argv = actual_compile['argv']
                compile_sources = {pathlib.Path(arg).resolve() for arg in argv if arg.endswith('.java')}
                expected_sources = {path.resolve() for path in generated_paths}
                require(compile_sources == expected_sources,
                        f'{leg}/{profile}: not every generated source, or an extra source, was compiled')
                require(pathlib.Path(argv[0]) == javac, f'{leg}/{profile}: wrong javac executable')
                require(ordered_subsequence(argv, fixture_flags),
                        f'{leg}/{profile}: generated javac flags/order differ from run-003')
                require('-g:none' in argv and '-classpath' in argv and '-sourcepath' in argv,
                        f'{leg}/{profile}: generated compile lacks isolated Java flags')
                require('-d' in argv and argv[argv.index('-d') + 1]
                        == str(BASELINE / 'generated' / leg / profile / 'classes'),
                        f'{leg}/{profile}: generated compile output directory differs')
                for flag in ('-classpath', '-sourcepath'):
                    idx = argv.index(flag) if flag in argv else -1
                    require(idx >= 0 and idx + 1 < len(argv) and pathlib.Path(argv[idx + 1]).is_dir(),
                            f'{leg}/{profile}: generated {flag} directory is missing')
                    if idx >= 0 and idx + 1 < len(argv):
                        require(empty_directory(pathlib.Path(argv[idx + 1])),
                                f'{leg}/{profile}: generated {flag} directory is not actually empty')
                        require(argv[idx + 1] == argv[argv.index('-classpath') + 1],
                                f'{leg}/{profile}: generated classpath/sourcepath differ')
                require(actual_compile['cwd'] == str(BASELINE / 'generated' / leg / profile),
                        f'{leg}/{profile}: generated compile cwd differs')
            require(case.get('compile_ok') is compile_ok, f'{leg}/{profile}: compile_ok differs from actual exit')

            runtime_record = case.get('runtime')
            actual_runtime = check_recorded_command(runtime_record, f'{leg}-{profile}-run') if runtime_record else None
            runtime_exit0 = actual_runtime is not None and actual_runtime['exit'] == 0
            streams_match = False
            exit_match = False
            if actual_runtime:
                argv = actual_runtime['argv']
                generated_classes = str(BASELINE / 'generated' / leg / profile / 'classes')
                require(pathlib.Path(argv[0]) == java, f'{leg}/{profile}: wrong java executable')
                require('-Xverify:all' in argv, f'{leg}/{profile}: candidate runtime lacks -Xverify:all')
                require('-cp' in argv and argv[argv.index('-cp') + 1] == generated_classes,
                        f'{leg}/{profile}: candidate runtime classpath is not its private generated classes')
                require(case.get('resolved_main_class') == argv[-1],
                        f'{leg}/{profile}: resolved main differs from runtime argv')
                candidate_stdout = evidence_path(actual_runtime['stdout']).read_bytes()
                candidate_stderr = evidence_path(actual_runtime['stderr']).read_bytes()
                streams_match = candidate_stdout == original_stdout and candidate_stderr == original_stderr
                exit_match = actual_runtime['exit'] == original_runtime['exit']
            require(case.get('candidate_runtime_exit0') is runtime_exit0,
                    f'{leg}/{profile}: candidate runtime exit-zero field differs')
            require(case.get('rawstreams_match') is streams_match,
                    f'{leg}/{profile}: rawstream match field differs from actual bytes')
            require(case.get('exit_match') is exit_match,
                    f'{leg}/{profile}: exit-match field differs from actual exits')

            original_compile_ok = leg_data.get('original_compile_ok') is True
            original_runtime_ok = leg_data.get('original_runtime_ok') is True
            expected_accepted = (original_compile_ok and original_runtime_ok and stage_render_ok
                                 and compile_ok and expected_quality and runtime_exit0
                                 and streams_match and exit_match)
            require(case.get('original_compile_ok') is original_compile_ok,
                    f'{leg}/{profile}: copied original_compile_ok differs')
            require(case.get('original_runtime_ok') is original_runtime_ok,
                    f'{leg}/{profile}: copied original_runtime_ok differs')
            require(case.get('accepted') is expected_accepted,
                    f'{leg}/{profile}: accepted field differs from independently recomputed gates')
            comparisons.append({
                'leg': leg, 'profile': profile, 'stage_render_ok': stage_render_ok,
                'markers': marker_summary, 'compile_exit': actual_compile['exit'] if actual_compile else None,
                'runtime_exit': actual_runtime['exit'] if actual_runtime else None,
                'runtime_exit0': runtime_exit0, 'rawstreams_equal': streams_match,
                'exit_equal': exit_match, 'quality_ok': expected_quality,
                'accepted_recomputed': expected_accepted,
            })

    result = {
        'schema': 'recover-constructor-primitive-conversion-baseline-root-verification-v1',
        'manifest_path': str(MANIFEST),
        'manifest_sha256': sha_file(MANIFEST),
        'comparison_legs_expected': 6,
        'comparison_legs_verified': len(comparisons),
        'comparisons': comparisons,
        'errors': errors,
        'verified': not errors and len(comparisons) == 6,
    }
    OUTPUT.write_text(json.dumps(result, indent=2, ensure_ascii=False) + '\n', encoding='utf-8')
    print(json.dumps({'output': str(OUTPUT), 'verified': result['verified'],
                      'errors': len(errors), 'comparison_legs': len(comparisons)}, indent=2))
    return 0 if result['verified'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
