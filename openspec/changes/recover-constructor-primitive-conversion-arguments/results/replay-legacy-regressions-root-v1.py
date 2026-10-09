#!/usr/bin/env python3
"""Re-render and replay the frozen legacy constructor/array regressions."""
from __future__ import annotations

import hashlib
import json
import pathlib
import re
import shutil
import subprocess
import sys
import zipfile

REPO = pathlib.Path('/Users/lordcasser/workspace/projects/jarde')
RESULTS = pathlib.Path(__file__).resolve().parent
COMPOSITION_RESULTS = REPO / 'openspec/changes/compose-constructed-reference-array-elements/results'
FAMILY_ROOT = COMPOSITION_RESULTS / 'legacy-family-cli2-root-v1'
MATRIX_ROOT = COMPOSITION_RESULTS / 'legacy-18-cli2-root-v1'
FAMILY_MANIFEST = FAMILY_ROOT / 'manifest.json'
MATRIX_MANIFEST = MATRIX_ROOT / 'manifest.json'
OUT = RESULTS / 'legacy-regressions-root-v1'


def sha_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha_file(path: pathlib.Path) -> str:
    return sha_bytes(path.read_bytes())


def record_file(path: pathlib.Path, root: pathlib.Path) -> dict:
    return {
        'path': str(path.relative_to(root)),
        'bytes': path.stat().st_size,
        'sha256': sha_file(path),
    }


def canonical_hash(value: object) -> str:
    return sha_bytes(json.dumps(value, sort_keys=True, separators=(',', ':'),
                                ensure_ascii=False).encode('utf-8'))


def command(label: str, argv: list[str], cwd: pathlib.Path,
            commands: list[dict]) -> tuple[subprocess.CompletedProcess[bytes], dict]:
    result = subprocess.run(argv, cwd=cwd, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    safe = re.sub(r'[^A-Za-z0-9_.-]+', '_', label)
    stdout_path = OUT / 'logs' / f'{safe}.stdout'
    stderr_path = OUT / 'logs' / f'{safe}.stderr'
    stdout_path.write_bytes(result.stdout)
    stderr_path.write_bytes(result.stderr)
    record = {
        'label': label,
        'argv': argv,
        'cwd': str(cwd),
        'exit': result.returncode,
        'stdout': str(stdout_path.relative_to(OUT)),
        'stdout_bytes': len(result.stdout),
        'stdout_sha256': sha_file(stdout_path),
        'stderr': str(stderr_path.relative_to(OUT)),
        'stderr_bytes': len(result.stderr),
        'stderr_sha256': sha_file(stderr_path),
    }
    commands.append(record)
    return result, record


def class_entries(jar_path: pathlib.Path) -> list[dict]:
    with zipfile.ZipFile(jar_path) as archive:
        rows = []
        for name in sorted(info.filename for info in archive.infolist()
                           if not info.is_dir() and info.filename.endswith('.class')):
            data = archive.read(name)
            rows.append({'entry': name, 'bytes': len(data), 'sha256': sha_bytes(data)})
        return rows


def raw_member_name(method: dict) -> str:
    raw = method.get('item', {}).get('name', {}).get('raw', [])
    return bytes(raw).decode('utf-8', errors='replace')


def source_map_facts(source_map: dict) -> dict:
    segments = source_map.get('segments', [])
    primary_bcis = set()
    all_bcis = set()
    derived = []
    for segment in segments:
        origin = segment.get('origin', {})
        primary = origin.get('primary', {})
        bci = primary.get('bci')
        if isinstance(bci, int):
            primary_bcis.add(bci)
            all_bcis.add(bci)
        for anchor in origin.get('derived', []):
            derived_bci = anchor.get('bci')
            if isinstance(derived_bci, int):
                all_bcis.add(derived_bci)
            method = anchor.get('method', {})
            derived.append({
                'bci': derived_bci,
                'cp': anchor.get('cp'),
                'provenance': anchor.get('provenance'),
                'method_name_bytes': method.get('name'),
                'method_descriptor_bytes': method.get('descriptor'),
                'owner_class_digest': method.get('owner', {}).get('class_bytes', {}).get('digest'),
            })
    return {
        'sha256': canonical_hash(source_map),
        'segment_count': len(segments),
        'primary_bcis': sorted(primary_bcis),
        'all_bcis': sorted(all_bcis),
        'derived_anchors': derived,
    }


def method_summaries(document: dict) -> list[dict]:
    summaries = []
    for method in document.get('methods', []):
        outcome = method.get('outcome', {})
        report = outcome.get('report') if outcome.get('kind') == 'recovered' else None
        body = report.get('text', '') if report else ''
        new_records = report.get('news', []) if report else []
        summaries.append({
            'name': raw_member_name(method),
            'outcome_kind': outcome.get('kind'),
            'analysis': outcome.get('analysis'),
            'outcome_refusal': outcome.get('refusal'),
            'member_markers': method.get('markers', []),
            'body_quality': report.get('quality') if report else None,
            'body_representation': report.get('representation') if report else None,
            'body_diagnostics': report.get('diagnostics') if report else None,
            'body_text_sha256': sha_bytes(body.encode('utf-8')) if report else None,
            'body_bytecode_marker': '@bytecode' in body if report else None,
            'body_refused_marker': 'jarde_refused_body' in body if report else None,
            'new_records': [{
                'class': record.get('class'),
                'head': record.get('head'),
                'dup': record.get('dup'),
                'constructor': record.get('constructor'),
                'arguments': record.get('arguments', []),
                'presented': record.get('presented'),
                'refusal': record.get('refusal'),
            } for record in new_records],
            'source_map': source_map_facts(report.get('source_map', {})) if report else None,
        })
    return summaries


def frozen_manifest_files(root: pathlib.Path, manifest: dict, label: str) -> None:
    """Verify the old result's own recorded artifacts before using its case data."""
    for item in manifest.get('files', []):
        path = root / item['path']
        if not path.is_file() or path.stat().st_size != item['bytes'] or sha_file(path) != item['sha256']:
            raise SystemExit(f'{label}: frozen artifact missing or changed: {item["path"]}')


def old_compile_command(manifest: dict, case: dict) -> dict:
    suffix = f'/{case["family"]}/{case["leg"]}/compile'
    matches = [item for item in manifest['commands'] if item['label'].endswith(suffix)]
    if len(matches) != 1:
        raise SystemExit(f'could not resolve one frozen compiler command for {case["family"]}/{case["leg"]}')
    return matches[0]


def main() -> int:
    if len(sys.argv) != 2:
        raise SystemExit('usage: replay-legacy-regressions-root-v1.py /absolute/path/to/frozen-cli')
    cli = pathlib.Path(sys.argv[1]).resolve()
    if not cli.is_file():
        raise SystemExit(f'candidate CLI does not exist: {cli}')
    if OUT.exists():
        raise SystemExit(f'refusing to overwrite legacy regression evidence: {OUT}')

    family_manifest = json.loads(FAMILY_MANIFEST.read_text(encoding='utf-8'))
    matrix_manifest = json.loads(MATRIX_MANIFEST.read_text(encoding='utf-8'))
    frozen_manifest_files(FAMILY_ROOT, family_manifest, 'legacy-family-cli2-root-v1')
    frozen_manifest_files(MATRIX_ROOT, matrix_manifest, 'legacy-18-cli2-root-v1')
    OUT.mkdir()
    (OUT / 'logs').mkdir()
    (OUT / 'inputs').mkdir()
    commands = []
    cases = []
    manifest = {
        'schema': 'compose-constructed-reference-array-elements-legacy-regressions-root-v1',
        'runner_path': str(pathlib.Path(__file__).resolve()),
        'runner_sha256': sha_file(pathlib.Path(__file__).resolve()),
        'cli_path': str(cli),
        'cli_sha256': sha_file(cli),
        'frozen_inputs': {
            'legacy_family_manifest': str(FAMILY_MANIFEST),
            'legacy_family_manifest_sha256': sha_file(FAMILY_MANIFEST),
            'legacy_18_manifest': str(MATRIX_MANIFEST),
            'legacy_18_manifest_sha256': sha_file(MATRIX_MANIFEST),
        },
        'original_stream_comparison_note': (
            'original stdout/stderr are compared to the frozen original_run exit and SHA-256 values; '
            'this replay does not rerun the original program or claim unavailable original raw bytes.'
        ),
        'cases': cases,
        'commands': commands,
    }

    jobs = []
    for dataset, old_root, old_manifest in (
        ('fixture', FAMILY_ROOT, family_manifest),
        ('baseline', MATRIX_ROOT, matrix_manifest),
    ):
        for case in old_manifest['cases']:
            jobs.append((dataset, old_root, old_manifest, case))
    if len(jobs) != 22:
        raise SystemExit(f'expected 22 frozen legacy cases; found {len(jobs)}')

    for index, (dataset, old_root, old_manifest, old_case) in enumerate(jobs):
        family = old_case['family']
        leg = old_case['leg']
        original_input = pathlib.Path(old_case['input_jar'])
        expected_jar_sha = old_case['input_jar_sha256']
        if not original_input.is_file() or sha_file(original_input) != expected_jar_sha:
            raise SystemExit(f'{dataset}/{family}/{leg}: frozen input JAR missing or SHA-256 mismatch')
        input_name = f'{index:02d}-{dataset}-{family}-{leg}.jar'
        copied_input = OUT / 'inputs' / input_name
        shutil.copyfile(original_input, copied_input)
        if sha_file(copied_input) != expected_jar_sha:
            raise SystemExit(f'{dataset}/{family}/{leg}: copied input JAR SHA-256 mismatch')
        entries = class_entries(copied_input)
        class_names = [entry['entry'][:-6].replace('/', '.') for entry in entries]
        if not class_names:
            raise SystemExit(f'{dataset}/{family}/{leg}: frozen input JAR has no .class entries')
        main_candidates = [name for name in class_names if name.rsplit('.', 1)[-1] in ('Main', 'CT')]
        if len(main_candidates) != 1:
            raise SystemExit(f'{dataset}/{family}/{leg}: cannot determine one main class from frozen JAR: {class_names}')
        main_class = main_candidates[0]

        compile_old = old_compile_command(old_manifest, old_case)
        javac = pathlib.Path(compile_old['argv'][0])
        java = javac.parent / 'java'
        try:
            classpath_index = compile_old['argv'].index('-classpath')
        except ValueError:
            raise SystemExit(f'{dataset}/{family}/{leg}: frozen compile command lacks -classpath')
        compiler_flags = compile_old['argv'][1:classpath_index]

        case_dir = OUT / dataset / family / leg
        sources = case_dir / 'sources'
        classes = case_dir / 'classes'
        empty = case_dir / 'empty-classpath-sourcepath'
        for directory in (sources, classes, empty):
            directory.mkdir(parents=True)

        prefix = f'{dataset}/{family}/{leg}'
        rendered_documents = []
        source_paths = []
        report_facts = []
        for class_name in class_names:
            rendered, rendered_record = command(
                f'{prefix}/render-{class_name}',
                [str(cli), 'class-source', '--input', str(copied_input), '--class', class_name,
                 '--policy', 'plain-jar', '--format', 'json', '--evidence', 'all', '--release', '8'],
                REPO, commands,
            )
            report = {
                'class': class_name,
                'command': rendered_record,
                'json_parsed': False,
                'source': None,
                'bytecode_marker': None,
                'refused_body_marker': None,
                'methods': [],
            }
            if rendered.returncode == 0:
                try:
                    document = json.loads(rendered.stdout)
                    text = document['text']
                    relative_source = pathlib.Path(*class_name.split('.')).with_suffix('.java')
                    source_path = sources / relative_source
                    source_path.parent.mkdir(parents=True, exist_ok=True)
                    source_path.write_text(text, encoding='utf-8')
                    source_paths.append(source_path)
                    rendered_documents.append(document)
                    report.update({
                        'json_parsed': True,
                        'report_class_identity': document.get('class'),
                        'declaration': document.get('declaration'),
                        'source': record_file(source_path, OUT),
                        'source_text_sha256': sha_bytes(text.encode('utf-8')),
                        'bytecode_marker': '@bytecode' in text,
                        'refused_body_marker': 'jarde_refused_body' in text,
                        'methods': method_summaries(document),
                    })
                except (json.JSONDecodeError, KeyError, TypeError, OSError) as error:
                    report['parse_or_write_error'] = str(error)
            report_facts.append(report)

        expected_source_paths = {pathlib.Path(*name.split('.')).with_suffix('.java').as_posix()
                                 for name in class_names}
        actual_source_paths = {path.relative_to(sources).as_posix() for path in source_paths}
        source_set_complete = (len(rendered_documents) == len(class_names)
                               and actual_source_paths == expected_source_paths)
        all_sources_without_refusal = (
            source_set_complete and all(item['json_parsed']
                                        and item['bytecode_marker'] is False
                                        and item['refused_body_marker'] is False
                                        for item in report_facts)
        )
        compile_result, compile_record = command(
            f'{prefix}/compile-candidate-sources',
            [str(javac), *compiler_flags, '-classpath', str(empty), '-sourcepath', str(empty),
             '-d', str(classes), *[str(path) for path in sorted(source_paths)]],
            case_dir, commands,
        )
        actual_class_files = sorted(classes.rglob('*.class'))
        candidate_class_names = sorted(path.relative_to(classes).as_posix() for path in actual_class_files)
        candidate_class_records = [record_file(path, OUT) for path in actual_class_files]

        old_original = old_case['original_run']
        runtime_record = None
        runtime_stdout_matches = False
        runtime_stderr_matches = False
        runtime_exit_matches = False
        runtime_exit_zero = False
        if compile_result.returncode == 0 and source_set_complete:
            runtime_result, runtime_record = command(
                f'{prefix}/run-candidate-classes',
                [str(java), '-Xverify:all', '-cp', str(classes), main_class],
                case_dir, commands,
            )
            runtime_stdout_matches = runtime_record['stdout_sha256'] == old_original['stdout_sha256']
            runtime_stderr_matches = runtime_record['stderr_sha256'] == old_original['stderr_sha256']
            runtime_exit_matches = runtime_result.returncode == old_original['exit']
            runtime_exit_zero = runtime_result.returncode == 0

        row = {
            'dataset': dataset,
            'family': family,
            'leg': leg,
            'old_case_input_jar': str(original_input),
            'old_case_input_jar_sha256': expected_jar_sha,
            'copied_input_jar': str(copied_input.relative_to(OUT)),
            'copied_input_jar_sha256': sha_file(copied_input),
            'input_class_entries': entries,
            'complete_input_class_names': class_names,
            'main_class': main_class,
            'compiler': str(javac),
            'runtime': str(java),
            'compiler_flags_from_old_case': compiler_flags,
            'prior_cli2_candidate': {
                'compile_exit': old_case.get('compile_exit'),
                'accepted': old_case.get('accepted'),
                'reports': [{
                    'class': report.get('class'),
                    'source_sha256': report.get('source_sha256'),
                    'bytecode_marker': report.get('bytecode_marker'),
                    'refused_body_marker': report.get('refused_body_marker'),
                    'report_stdout': report.get('report_stdout'),
                } for report in old_case.get('reports', [])],
            },
            'candidate_source_reports': report_facts,
            'source_set_complete': source_set_complete,
            'all_sources_without_bytecode_or_refused_body_markers': all_sources_without_refusal,
            'candidate_compile': compile_record,
            'candidate_compile_class_set': candidate_class_names,
            'candidate_classes': candidate_class_records,
            'candidate_runtime': runtime_record,
            'frozen_original_run': old_original,
            'original_raw_streams_available': False,
            'candidate_stdout_hash_matches_original_record': runtime_stdout_matches,
            'candidate_stderr_hash_matches_original_record': runtime_stderr_matches,
            'candidate_exit_matches_original_record': runtime_exit_matches,
            'candidate_exit_zero': runtime_exit_zero,
            'matches_frozen_original_run': (runtime_stdout_matches and runtime_stderr_matches
                                            and runtime_exit_matches),
        }
        row['candidate_success'] = (
            source_set_complete and all_sources_without_refusal
            and compile_record['exit'] == 0
            and runtime_exit_zero and row['matches_frozen_original_run']
        )
        cases.append(row)
        manifest['completed_case_count'] = len(cases)
        (OUT / 'manifest.json').write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + '\n',
                                           encoding='utf-8')

    manifest['files'] = [record_file(path, OUT) for path in sorted(OUT.rglob('*'))
                         if path.is_file() and path != OUT / 'manifest.json']
    (OUT / 'manifest.json').write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + '\n',
                                       encoding='utf-8')
    print(json.dumps({
        'manifest': str((OUT / 'manifest.json').relative_to(REPO)),
        'runner_sha256': manifest['runner_sha256'],
        'cli_path': manifest['cli_path'],
        'cli_sha256': manifest['cli_sha256'],
        'case_count': len(cases),
        'candidate_success_count': sum(bool(case['candidate_success']) for case in cases),
    }, indent=2, ensure_ascii=False))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
