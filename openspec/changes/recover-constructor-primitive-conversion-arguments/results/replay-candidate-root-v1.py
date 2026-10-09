#!/usr/bin/env python3
"""Replay the frozen two-class inputs through a caller-supplied candidate CLI."""
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
BASELINE = RESULTS / 'baseline-v1'
BASELINE_MANIFEST = BASELINE / 'manifest.json'
OUT = RESULTS / 'candidate-root-v1'
SOURCE_NAMES = ('ConstructorPrimitiveConversionControls.java', 'PrimitiveLongPair.java')
CLASS_NAMES = ('ConstructorPrimitiveConversionControls', 'PrimitiveLongPair')
MAIN = CLASS_NAMES[0]

TARGET_CLASSES = {
    'wrapperByte': ['java/lang/Byte'], 'wrapperShort': ['java/lang/Short'],
    'wrapperLong': ['java/lang/Long'], 'wrapperFloat': ['java/lang/Float'],
    'wrapperDouble': ['java/lang/Double'], 'integerContrast': ['java/lang/Integer'],
    'boxedArray': ['java/lang/Byte', 'java/lang/Short', 'java/lang/Integer',
                   'java/lang/Long', 'java/lang/Float', 'java/lang/Double'],
    'ordinaryReturnNew': ['java/lang/Long'], 'storedLocalReuse': ['PrimitiveLongPair'],
    'intToLong': ['java/lang/Long'], 'intToFloat': ['java/lang/Float'],
    'intToDouble': ['java/lang/Double'], 'intToByte': ['java/lang/Byte'],
    'intToCharacter': ['java/lang/Character'], 'intToShort': ['java/lang/Short'],
    'longToInt': ['java/lang/Integer'], 'longToFloat': ['java/lang/Float'],
    'longToDouble': ['java/lang/Double'], 'floatToInt': ['java/lang/Integer'],
    'floatToLong': ['java/lang/Long'], 'floatToDouble': ['java/lang/Double'],
    'doubleToInt': ['java/lang/Integer'], 'doubleToLong': ['java/lang/Long'],
    'doubleToFloat': ['java/lang/Float'], 'longViaFloat': ['java/lang/Long'],
    'longViaDouble': ['java/lang/Long'], 'doubleViaFloat': ['java/lang/Double'],
}
CASTS = {
    'wrapperByte': [('byte', 10)], 'wrapperShort': [('short', 10)],
    'wrapperLong': [('long', 10)], 'wrapperFloat': [('float', 10)],
    'wrapperDouble': [('double', 10)],
    'boxedArray': [('byte', 17), ('short', 34), ('long', 67), ('float', 84), ('double', 101)],
    'ordinaryReturnNew': [('long', 10)],
    'storedLocalReuse': [('long', 12), ('long', 14)],
    'intToLong': [('long', 10)], 'intToFloat': [('float', 10)],
    'intToDouble': [('double', 10)], 'intToByte': [('byte', 10)],
    'intToCharacter': [('char', 10)], 'intToShort': [('short', 10)],
    'longToInt': [('int', 10)], 'longToFloat': [('float', 10)],
    'longToDouble': [('double', 10)], 'floatToInt': [('int', 10)],
    'floatToLong': [('long', 10)], 'floatToDouble': [('double', 10)],
    'doubleToInt': [('int', 10)], 'doubleToLong': [('long', 10)],
    'doubleToFloat': [('float', 10)], 'longViaFloat': [('long', 11), ('float', 10)],
    'longViaDouble': [('long', 11), ('double', 10)],
    'doubleViaFloat': [('double', 11), ('float', 10)],
}
ARG_COUNTS = {'storedLocalReuse': [2]}
EXTRA_BCIS = {'boxedArray': [21, 38, 54, 71, 88, 105], 'storedLocalReuse': [6]}
CALLS = {
    'wrapperFloat': 'markLong', 'wrapperDouble': 'markFloat',
    'longToInt': 'markLong', 'longToFloat': 'markLong', 'longToDouble': 'markLong',
    'floatToInt': 'markFloat', 'floatToLong': 'markFloat', 'floatToDouble': 'markFloat',
    'doubleToInt': 'markDouble', 'doubleToLong': 'markDouble', 'doubleToFloat': 'markDouble',
    'longViaFloat': 'markLong', 'longViaDouble': 'markLong', 'doubleViaFloat': 'markDouble',
}
CALL_COUNTS = {'boxedArray': 6}
ARRAY_CLASS = 'java/lang/Number'
ARRAY_AASTORE_BCIS = [21, 38, 54, 71, 88, 105]


def sha_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha_file(path: pathlib.Path) -> str:
    return sha_bytes(path.read_bytes())


def source_record(path: pathlib.Path, root: pathlib.Path) -> dict:
    return {'path': str(path.relative_to(root)), 'bytes': path.stat().st_size, 'sha256': sha_file(path)}


def canonical_hash(value: object) -> str:
    return sha_bytes(json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode())


def raw_method_name(method: dict) -> str:
    raw = method['item']['name']['raw']
    return bytes(raw).decode('utf-8', errors='replace')


def marker_facts(text: str) -> dict:
    return {
        'bytecode_marker': '@bytecode' in text,
        'refused_body_marker': 'jarde_refused_body' in text,
    }


def summarize_source_map(source_map: dict) -> dict:
    segments = source_map.get('segments', [])
    anchors = []
    primary_bcis = set()
    all_bcis = set()
    for segment in segments:
        origin = segment.get('origin', {})
        primary = origin.get('primary', {})
        bci = primary.get('bci')
        if isinstance(bci, int):
            primary_bcis.add(bci)
            all_bcis.add(bci)
        derived_anchors = []
        derived_bcis = set()
        for derived in origin.get('derived', []):
            derived_bci = derived.get('bci')
            if isinstance(derived_bci, int):
                derived_bcis.add(derived_bci)
                all_bcis.add(derived_bci)
            derived_method = derived.get('method', {})
            derived_anchors.append({
                'bci': derived_bci, 'cp': derived.get('cp'),
                'provenance': derived.get('provenance'),
                'method_name_bytes': derived_method.get('name'),
                'method_descriptor_bytes': derived_method.get('descriptor'),
                'owner_class_digest': (derived_method.get('owner', {}).get('class_bytes', {}).get('digest')),
            })
        anchors.append({
            'start': segment.get('start'), 'end': segment.get('end'),
            'primary_bci': bci, 'primary_cp': primary.get('cp'),
            'provenance': primary.get('provenance'),
            'owner_class_digest': (primary.get('method', {}).get('owner', {})
                                   .get('class_bytes', {}).get('digest')),
            'derived_count': len(derived_anchors),
            'derived_bcis': sorted(derived_bcis),
            'derived_anchors': derived_anchors,
        })
    return {
        'sha256': canonical_hash(source_map),
        'segment_count': len(segments),
        'primary_bcis': sorted(primary_bcis),
        'all_bcis': sorted(all_bcis),
        'segments': anchors,
    }


def validate_frozen_baseline_leg(leg: str, original: dict, baseline_manifest: dict) -> dict:
    expected_source_names = set(SOURCE_NAMES)
    expected_class_files = {f'{name}.class' for name in CLASS_NAMES}
    source_root = REPO / 'openspec/changes/recover-constructor-primitive-conversion-arguments/results/draft-sources-v1'
    closure_records = baseline_manifest['source_closure']
    closure_by_name = {pathlib.Path(record['path']).name: record for record in closure_records}
    if set(closure_by_name) != expected_source_names:
        raise SystemExit(f'{leg}: frozen source closure does not equal expected source names')

    source_records = original['original_sources']
    class_records = original['original_classes']
    source_paths = [BASELINE / record['path'] for record in source_records]
    class_paths = [BASELINE / record['path'] for record in class_records]
    if {path.name for path in source_paths} != expected_source_names:
        raise SystemExit(f'{leg}: original source records do not cover the complete source set')
    if {path.name for path in class_paths} != expected_class_files:
        raise SystemExit(f'{leg}: original class records do not cover the complete class set')

    def check_record(path: pathlib.Path, record: dict, label: str) -> bytes:
        if not path.is_file():
            raise SystemExit(f'{leg}: frozen {label} is missing: {path}')
        data = path.read_bytes()
        if len(data) != record['bytes'] or sha_bytes(data) != record['sha256']:
            raise SystemExit(f'{leg}: frozen {label} bytes/hash mismatch: {path}')
        return data

    original_source_bytes = {}
    for path, record in zip(source_paths, source_records):
        data = check_record(path, record, 'original source')
        original_source_bytes[path.name] = data
        closure = closure_by_name[path.name]
        closure_path = REPO / closure['path']
        closure_data = check_record(closure_path, closure, 'draft source closure')
        if data != closure_data:
            raise SystemExit(f'{leg}: copied original source differs from frozen draft closure: {path.name}')

    source_dir = source_paths[0].parent
    if {path.relative_to(source_dir).as_posix() for path in source_dir.rglob('*.java')} != expected_source_names:
        raise SystemExit(f'{leg}: original source directory has an unexpected .java closure')
    if {path.relative_to(source_root).as_posix() for path in source_root.rglob('*.java')} != expected_source_names:
        raise SystemExit(f'{leg}: draft source directory has an unexpected .java closure')

    original_class_bytes = {}
    for path, record in zip(class_paths, class_records):
        original_class_bytes[path.name] = check_record(path, record, 'original class')
    class_dir = class_paths[0].parent
    if {path.relative_to(class_dir).as_posix() for path in class_dir.rglob('*.class')} != expected_class_files:
        raise SystemExit(f'{leg}: original class directory has an unexpected .class closure')

    jar_record = original['original_input_jar']
    jar_path = BASELINE / jar_record['path']
    if not jar_path.is_file() or sha_file(jar_path) != jar_record['sha256']:
        raise SystemExit(f'{leg}: frozen baseline input JAR is missing or changed')
    declared_entries = {entry['name']: entry for entry in jar_record['class_entries']}
    if set(declared_entries) != expected_class_files:
        raise SystemExit(f'{leg}: JAR manifest entries do not cover the complete class set')
    with zipfile.ZipFile(jar_path) as archive:
        actual_names = [name for name in archive.namelist() if not name.endswith('/')]
        if set(actual_names) != expected_class_files or len(actual_names) != len(expected_class_files):
            raise SystemExit(f'{leg}: frozen JAR has an unexpected entry closure')
        jar_entries = {}
        for name, record in declared_entries.items():
            data = archive.read(name)
            if len(data) != record['bytes'] or sha_bytes(data) != record['sha256']:
                raise SystemExit(f'{leg}: JAR entry bytes/hash mismatch: {name}')
            if data != original_class_bytes[name]:
                raise SystemExit(f'{leg}: JAR entry differs from original class file: {name}')
            jar_entries[name] = {'bytes': len(data), 'sha256': sha_bytes(data)}
    return {
        'source_closure_names': sorted(expected_source_names),
        'source_closure_verified': True,
        'original_source_file_hashes_verified': True,
        'original_class_closure_names': sorted(expected_class_files),
        'original_class_file_hashes_verified': True,
        'jar_entry_closure_names': sorted(expected_class_files),
        'jar_entries_match_original_class_files': True,
        'jar_entry_hashes': jar_entries,
    }


def write_command(label: str, argv: list[str], cwd: pathlib.Path, out: pathlib.Path,
                  commands: list[dict]) -> tuple[subprocess.CompletedProcess[bytes], dict]:
    result = subprocess.run(argv, cwd=cwd, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    safe = re.sub(r'[^A-Za-z0-9_.-]+', '_', label)
    stdout_path = out / 'logs' / f'{safe}.stdout'
    stderr_path = out / 'logs' / f'{safe}.stderr'
    stdout_path.write_bytes(result.stdout)
    stderr_path.write_bytes(result.stderr)
    record = {
        'label': label, 'argv': argv, 'cwd': str(cwd), 'exit': result.returncode,
        'stdout': str(stdout_path.relative_to(out)), 'stdout_bytes': len(result.stdout),
        'stdout_sha256': sha_file(stdout_path), 'stderr': str(stderr_path.relative_to(out)),
        'stderr_bytes': len(result.stderr), 'stderr_sha256': sha_file(stderr_path),
    }
    commands.append(record)
    return result, record


def summarize_methods(document: dict) -> tuple[list[dict], bool]:
    method_facts = []
    all_markers_clear = not any(marker_facts(document.get('text', '')).values())
    for method in document.get('methods', []):
        name = raw_method_name(method)
        member_text = method.get('text', '')
        member_markers = method.get('markers', [])
        member_marker_facts = marker_facts(member_text)
        outcome = method.get('outcome', {})
        recovery = outcome.get('report') if outcome.get('kind') == 'recovered' else None
        body_text = recovery.get('text', '') if recovery else ''
        body_markers = marker_facts(body_text)
        all_markers_clear = (all_markers_clear and not member_markers
                             and not any(member_marker_facts.values())
                             and not any(body_markers.values()))
        source_map = recovery.get('source_map', {}) if recovery else {}
        new_records = recovery.get('news', []) if recovery else []
        method_facts.append({
            'name': name,
            'descriptor_bytes': method.get('item', {}).get('descriptor', {}).get('raw'),
            'member_markers': member_markers,
            'member_text_markers': member_marker_facts,
            'outcome_kind': outcome.get('kind'),
            'body_quality': recovery.get('quality') if recovery else None,
            'body_representation': recovery.get('representation') if recovery else None,
            'body_markers': body_markers if recovery else None,
            'body_text_sha256': sha_bytes(body_text.encode()) if recovery else None,
            'new_records': new_records,
            'source_map_identity': summarize_source_map(source_map) if recovery else None,
        })
    return method_facts, all_markers_clear


def check_targets(document: dict, method_facts: list[dict]) -> dict:
    by_name = {method['name']: method for method in method_facts}
    total_sites = 0
    site_ids = []
    details = []
    all_target_bodies_ok = True
    for name, expected_classes in TARGET_CLASSES.items():
        member = by_name.get(name)
        if member is None:
            details.append({'method': name, 'present': False, 'site_count': 0, 'ok': False})
            all_target_bodies_ok = False
            continue
        records = member['new_records'] or []
        actual_classes = [record.get('class') for record in records]
        expected_args = ARG_COUNTS.get(name, [1] * len(expected_classes))
        source_map = member.get('source_map_identity') or {}
        source_bcis = set(source_map.get('all_bcis', []))
        site_map_ok = True
        for record in records:
            for bci in [record.get('head'), record.get('dup'), record.get('constructor'), *record.get('arguments', [])]:
                if isinstance(bci, int) and bci not in source_bcis:
                    site_map_ok = False
        casts = CASTS.get(name, [])
        body_text_sha = member.get('body_text_sha256')
        # The full body is retained in the raw class-source JSON stream; the summary records
        # target cast BCIs and site/source-map identities without duplicating that source.
        report_method = next((item for item in document.get('methods', []) if raw_method_name(item) == name), None)
        recovery = report_method.get('outcome', {}).get('report') if report_method else None
        body_text = recovery.get('text', '') if recovery else ''
        cast_ok = all(f'({primitive})' in body_text and bci in source_bcis for primitive, bci in casts)
        ordered_casts = []
        search_cursor = 0
        for primitive, _ in casts:
            token = f'({primitive})'
            position = body_text.find(token, search_cursor)
            ordered_casts.append(position)
            if position >= 0:
                search_cursor = position + len(token)
        cast_order_ok = all(left < right for left, right in zip(ordered_casts, ordered_casts[1:]))
        extra_bci_ok = all(bci in source_bcis for bci in EXTRA_BCIS.get(name, []))
        call_name = CALLS.get(name, 'markInt')
        call_count = body_text.count(f'{call_name}(')
        expected_call_count = CALL_COUNTS.get(name, 1)
        body_ok = (
            member['outcome_kind'] == 'recovered'
            and member['body_quality'] == 'structured'
            and member['body_representation'] == 'java'
            and not any((member['member_markers'] or []))
            and not any((member['member_text_markers'] or {}).values())
            and not any((member['body_markers'] or {}).values())
            and len(records) == len(expected_classes)
            and actual_classes == expected_classes
            and all(record.get('presented') is True and record.get('refusal') is None for record in records)
            and [len(record.get('arguments', [])) for record in records] == expected_args
            and len({record.get('head') for record in records}) == len(records)
            and site_map_ok and cast_ok and cast_order_ok and extra_bci_ok
            and call_count == expected_call_count
        )
        total_sites += len(records)
        site_ids.extend((name, record.get('head')) for record in records)
        details.append({
            'method': name, 'present': True, 'outcome_kind': member['outcome_kind'],
            'quality': member['body_quality'], 'representation': member['body_representation'],
            'expected_new_classes': expected_classes, 'actual_new_classes': actual_classes,
            'presented': [record.get('presented') for record in records],
            'refusals': [record.get('refusal') for record in records],
            'new_count': len(records), 'source_map_sha256': source_map.get('sha256'),
            'source_map_primary_bcis': source_map.get('primary_bcis', []),
            'source_map_all_bcis': source_map.get('all_bcis', []),
            'site_map_ok': site_map_ok, 'cast_evidence': [{'primitive': primitive, 'bci': bci,
                                                            'text_present': f'({primitive})' in body_text,
                                                            'mapped': bci in source_bcis}
                                                           for primitive, bci in casts],
            'cast_order_ok': cast_order_ok, 'extra_bcis_ok': extra_bci_ok,
            'call_name': call_name, 'call_count': call_count,
            'body_text_sha256': body_text_sha, 'ok': body_ok,
        })
        all_target_bodies_ok = all_target_bodies_ok and body_ok

    stored = by_name.get('storedLocalReuse')
    stored_report_method = next((item for item in document.get('methods', [])
                                 if raw_method_name(item) == 'storedLocalReuse'), None)
    stored_recovery = stored_report_method.get('outcome', {}).get('report') if stored_report_method else None
    stored_once = bool(stored_recovery) and stored_recovery.get('text', '').count('markInt(') == 1
    site_ids_unique = len(site_ids) == len(set(site_ids))
    all_target_bodies_ok = all_target_bodies_ok and total_sites == 32 and site_ids_unique and stored_once
    return {
        'target_methods_expected': len(TARGET_CLASSES),
        'target_methods': details,
        'new_record_count': total_sites,
        'new_record_site_identities_unique': site_ids_unique,
        'stored_local_producer_occurs_once': stored_once,
        'all_target_bodies_ok': all_target_bodies_ok,
        'array_class': ARRAY_CLASS,
        'array_store_bcis': ARRAY_AASTORE_BCIS,
        'array_store_bcis_mapped': all(bci in set((by_name.get('boxedArray') or {}).get('source_map_identity', {}).get('all_bcis', []))
                                       for bci in ARRAY_AASTORE_BCIS),
    }


def main() -> int:
    if len(sys.argv) != 2:
        raise SystemExit('usage: replay-candidate-root-v1.py /absolute/path/to/frozen-cli')
    cli = pathlib.Path(sys.argv[1]).resolve()
    if not cli.is_file():
        raise SystemExit(f'candidate CLI does not exist: {cli}')
    if OUT.exists():
        raise SystemExit(f'refusing to overwrite candidate evidence directory: {OUT}')
    baseline_manifest = json.loads(BASELINE_MANIFEST.read_text(encoding='utf-8'))
    OUT.mkdir()
    (OUT / 'logs').mkdir()
    (OUT / 'generated').mkdir()
    commands = []
    cases = []
    manifest = {
        'schema': 'recover-constructor-primitive-conversion-candidate-root-v1',
        'runner_path': str(pathlib.Path(__file__).resolve()),
        'runner_sha256': sha_file(pathlib.Path(__file__).resolve()),
        'baseline_manifest_path': str(BASELINE_MANIFEST),
        'baseline_manifest_sha256': sha_file(BASELINE_MANIFEST),
        'baseline_scope': 'two original source/class pairs and original dual streams are immutable references; no JADX rerun',
        'cli_path': str(cli),
        'cli_sha256': sha_file(cli),
        'jdk_legs': {},
        'commands': commands,
        'cases': cases,
    }

    for leg, original in baseline_manifest['jdk_legs'].items():
        home = pathlib.Path(original['jdk_home'])
        javac = home / 'bin/javac'
        java = home / 'bin/java'
        frozen_input_checks = validate_frozen_baseline_leg(leg, original, baseline_manifest)
        original_jar_record = original['original_input_jar']
        input_jar = BASELINE / original_jar_record['path']
        runtime = original['original_runtime']
        original_stdout = BASELINE / runtime['stdout']
        original_stderr = BASELINE / runtime['stderr']
        if (not original_stdout.is_file() or not original_stderr.is_file()
                or sha_file(original_stdout) != runtime['stdout_sha256']
                or sha_file(original_stderr) != runtime['stderr_sha256']):
            raise SystemExit(f'{leg}: frozen original runtime streams are missing or changed')
        manifest['jdk_legs'][leg] = {
            'jdk_home': str(home),
            'compiler_flags_from_baseline_manifest': original['compiler_flags_from_fixture_v1_run003'],
            'javac_sha256': sha_file(javac),
            'java_sha256': sha_file(java),
            'baseline_input_jar': original_jar_record,
            'baseline_frozen_input_checks': frozen_input_checks,
            'baseline_original_class_hashes': original['original_classes'],
            'baseline_original_sources': original['original_sources'],
            'baseline_original_runtime': runtime,
            'candidate_source_reports': [],
            'candidate_compile': None,
            'candidate_compile_ok': False,
            'candidate_runtime': None,
            'candidate_runtime_ok': False,
        }
        work = OUT / 'generated' / leg
        sources = work / 'sources'
        classes = work / 'classes'
        empty = work / 'empty-classpath-sourcepath'
        sources.mkdir(parents=True)
        classes.mkdir()
        empty.mkdir()
        documents = []
        source_paths = []

        for class_name in CLASS_NAMES:
            argv = [str(cli), 'class-source', '--input', str(input_jar), '--class', class_name,
                    '--policy', 'plain-jar', '--format', 'json', '--evidence', 'all', '--release', '8']
            rendered, rendered_record = write_command(
                f'{leg}-render-{class_name}', argv, REPO, OUT, commands)
            report_facts = {
                'class_name': class_name,
                'command': rendered_record,
                'json_parsed': False,
                'source': None,
                'class_markers': None,
                'member_summaries': [],
                'all_markers_clear': False,
                'target_checks': None,
            }
            if rendered.returncode == 0:
                try:
                    document = json.loads(rendered.stdout)
                    source_text = document['text']
                    source = sources / f'{class_name}.java'
                    source.write_text(source_text, encoding='utf-8')
                    source_paths.append(source)
                    documents.append(document)
                    member_summaries, all_markers_clear = summarize_methods(document)
                    report_facts.update({
                        'json_parsed': True,
                        'report_class_identity': document.get('class'),
                        'declaration': document.get('declaration'),
                        'source': source_record(source, OUT),
                        'class_markers': marker_facts(source_text),
                        'member_summaries': member_summaries,
                        'all_markers_clear': all_markers_clear,
                        'source_text_sha256': sha_bytes(source_text.encode()),
                    })
                    if class_name == MAIN:
                        report_facts['target_checks'] = check_targets(document, member_summaries)
                except (json.JSONDecodeError, KeyError, TypeError) as error:
                    report_facts['json_parse_error'] = str(error)
            manifest['jdk_legs'][leg]['candidate_source_reports'].append(report_facts)

        source_set_complete = (len(source_paths) == len(SOURCE_NAMES)
                               and {path.name for path in source_paths} == set(SOURCE_NAMES)
                               and len(documents) == len(CLASS_NAMES))
        all_source_markers_clear = (source_set_complete and len(documents) == len(CLASS_NAMES)
                                    and all(report['all_markers_clear'] for report in manifest['jdk_legs'][leg]['candidate_source_reports']))
        main_document = next((document for document in documents
                              if (document.get('declaration') or {}).get('name') == MAIN), None)
        main_report_record = next((report for report in manifest['jdk_legs'][leg]['candidate_source_reports']
                                   if report['class_name'] == MAIN), None)
        target_facts = (check_targets(main_document, main_report_record['member_summaries'])
                        if main_document is not None and main_report_record is not None
                        else {'all_target_bodies_ok': False, 'new_record_count': 0,
                              'new_record_site_identities_unique': False, 'target_methods': [],
                              'array_store_bcis_mapped': False})
        manifest['jdk_legs'][leg]['source_set_complete'] = source_set_complete
        manifest['jdk_legs'][leg]['all_source_member_body_markers_clear'] = all_source_markers_clear
        manifest['jdk_legs'][leg]['target_checks'] = target_facts

        compile_result, compile_record = write_command(
            f'{leg}-compile-candidate-sources',
            [str(javac), *original['compiler_flags_from_fixture_v1_run003'], '-g:none',
             '-classpath', str(empty), '-sourcepath', str(empty), '-d', str(classes),
             *[str(path) for path in sorted(source_paths)]],
            work, OUT, commands,
        )
        compile_ok = compile_result.returncode == 0 and source_set_complete
        candidate_class_files = sorted(classes.rglob('*.class'))
        candidate_class_records = [source_record(path, OUT) for path in candidate_class_files]
        candidate_class_set = sorted(path.name for path in candidate_class_files)
        class_set_complete = candidate_class_set == sorted(f'{name}.class' for name in CLASS_NAMES)
        compile_ok = compile_ok and class_set_complete
        manifest['jdk_legs'][leg]['candidate_compile'] = compile_record
        manifest['jdk_legs'][leg]['candidate_compile_ok'] = compile_ok
        manifest['jdk_legs'][leg]['candidate_class_set'] = candidate_class_set
        manifest['jdk_legs'][leg]['candidate_classes'] = candidate_class_records
        runtime_record = None
        candidate_runtime_ok = False
        streams_match = False
        exit_match = False
        if compile_result.returncode == 0:
            runtime_result, runtime_record = write_command(
                f'{leg}-run-candidate-classes',
                [str(java), '-Xverify:all', '-cp', str(classes), MAIN],
                work, OUT, commands,
            )
            candidate_runtime_ok = runtime_result.returncode == 0
            streams_match = runtime_result.stdout == original_stdout.read_bytes() and runtime_result.stderr == original_stderr.read_bytes()
            exit_match = runtime_result.returncode == runtime['exit']
        manifest['jdk_legs'][leg]['candidate_runtime'] = runtime_record
        manifest['jdk_legs'][leg]['candidate_runtime_ok'] = candidate_runtime_ok
        manifest['jdk_legs'][leg]['rawstreams_match_original'] = streams_match
        manifest['jdk_legs'][leg]['exit_match_original'] = exit_match
        manifest['jdk_legs'][leg]['accepted'] = (
            source_set_complete and all_source_markers_clear and target_facts.get('all_target_bodies_ok', False)
            and target_facts.get('new_record_count') == 32
            and target_facts.get('array_store_bcis_mapped') is True
            and compile_ok and candidate_runtime_ok and streams_match and exit_match
        )
        manifest['cases'].append({
            'leg': leg, 'source_set_complete': source_set_complete,
            'all_source_member_body_markers_clear': all_source_markers_clear,
            'target_checks': target_facts, 'compile_exit': compile_record['exit'],
            'runtime_exit': runtime_record['exit'] if runtime_record else None,
            'runtime_ok': candidate_runtime_ok, 'rawstreams_match_original': streams_match,
            'exit_match_original': exit_match, 'accepted': manifest['jdk_legs'][leg]['accepted'],
        })
        save_manifest(manifest)

    manifest['files'] = [
        source_record(path, OUT) for path in sorted(OUT.rglob('*'))
        if path.is_file() and path != OUT / 'manifest.json'
    ]
    save_manifest(manifest)
    print(json.dumps({
        'manifest': str((OUT / 'manifest.json').relative_to(REPO)),
        'runner_sha256': manifest['runner_sha256'], 'cli_path': manifest['cli_path'],
        'cli_sha256': manifest['cli_sha256'], 'baseline_manifest_sha256': manifest['baseline_manifest_sha256'],
        'cases': manifest['cases'],
        'accepted_cases': sum(bool(case['accepted']) for case in manifest['cases']),
    }, indent=2, ensure_ascii=False))
    return 0


def save_manifest(manifest: dict) -> None:
    (OUT / 'manifest.json').write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + '\n', encoding='utf-8')


if __name__ == '__main__':
    raise SystemExit(main())
