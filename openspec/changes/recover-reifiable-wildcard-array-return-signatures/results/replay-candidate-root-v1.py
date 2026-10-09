#!/usr/bin/env python3
"""Freshly render and replay the frozen wildcard-array candidate matrix.

Usage: replay-candidate-root-v1.py FROZEN_CLI CLI_METADATA_JSON ABSOLUTE_OUTPUT_DIR
This script is intentionally not run while being prepared. It uses the real frozen
JDKs and jars, and preserves all raw command streams in the new output directory.
"""
from __future__ import annotations

import hashlib
import json
import os
import pathlib
import re
import shutil
import subprocess
import sys
import tarfile
import zipfile

REPO = pathlib.Path('/Users/lordcasser/workspace/projects/jarde')
COMPOSITION = REPO / 'openspec/changes/compose-constructed-reference-array-elements/results'
FAMILY_ROOT = COMPOSITION / 'legacy-family-cli2-root-v1'
MATRIX_ROOT = COMPOSITION / 'legacy-18-cli2-root-v1'
NUMERIC_RESULTS = REPO / 'openspec/changes/recover-constructor-primitive-conversion-arguments/results'
NUMERIC_ROOT = NUMERIC_RESULTS / 'baseline-v1'
BIGDECIMAL_ROOT = NUMERIC_RESULTS / 'bigdecimal-historical-provenance-v1'
FAMILY_MANIFEST_PATH = FAMILY_ROOT / 'manifest.json'
MATRIX_MANIFEST_PATH = MATRIX_ROOT / 'manifest.json'
NUMERIC_MANIFEST_PATH = NUMERIC_ROOT / 'manifest.json'
BIGDECIMAL_MANIFEST_PATH = BIGDECIMAL_ROOT / 'manifest.json'
BASELINE_ARCHIVE = REPO / 'openspec/changes/recover-heterogeneous-array-init/evidence/baseline-20261009.tar.gz'
BASELINE_ARCHIVE_METADATA = REPO / 'openspec/changes/recover-heterogeneous-array-init/results/baseline-archive.json'
BASELINE_REFERENCE_AUDIT_PATH = REPO / 'openspec/changes/recover-covariant-child-array-initializers/results/baseline-reference-audit-v1.json'
BASELINE_ROOT_VERIFICATION_PATH = REPO / 'openspec/changes/recover-covariant-child-array-initializers/results/baseline-root-verification-v2.json'
GENERIC_BASELINE_SUMMARY_PATH = REPO / 'openspec/changes/recover-reifiable-wildcard-array-return-signatures/results/baseline-root-v1.json'
REQUIRED_CANDIDATE_SOURCES = {
    'crates/jarde-java/src/init.rs',
    'crates/jarde-java/src/report.rs',
    'crates/jarde-java/src/build.rs',
    'src/class_source.rs',
    'Cargo.lock',
}
WORKSPACE = pathlib.Path('/Users/lordcasser/workspace/projects/jarde')


def sha_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha_file(path: pathlib.Path) -> str:
    return sha_bytes(path.read_bytes())


def file_record(path: pathlib.Path, root: pathlib.Path) -> dict:
    return {'path': path.relative_to(root).as_posix(), 'bytes': path.stat().st_size,
            'sha256': sha_file(path)}


def canonical_hash(value: object) -> str:
    return sha_bytes(json.dumps(value, sort_keys=True, separators=(',', ':'),
                                ensure_ascii=False).encode('utf-8'))


def verify_frozen_files(root: pathlib.Path, manifest: dict, label: str) -> None:
    for row in manifest.get('files', []):
        path = root / row['path']
        if not path.is_file() or path.stat().st_size != row['bytes'] or sha_file(path) != row['sha256']:
            raise SystemExit(f'{label}: frozen evidence missing or changed: {path}')


def class_entries(jar: pathlib.Path) -> list[dict]:
    rows = []
    with zipfile.ZipFile(jar) as archive:
        for name in sorted(info.filename for info in archive.infolist()
                           if not info.is_dir() and info.filename.endswith('.class')):
            data = archive.read(name)
            rows.append({'entry': name, 'bytes': len(data), 'sha256': sha_bytes(data)})
    return rows


def command(label: str, argv: list[str], cwd: pathlib.Path,
            out: pathlib.Path, commands: list[dict]) -> tuple[subprocess.CompletedProcess[bytes], dict]:
    # Avoid ambient JVM injection while giving every subprocess the same explicit policy.
    env = os.environ.copy()
    cleared = {}
    for name in ('JAVA_TOOL_OPTIONS', '_JAVA_OPTIONS', 'JDK_JAVA_OPTIONS'):
        cleared[name] = name in env
        env.pop(name, None)
    result = subprocess.run(argv, cwd=cwd, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    safe = re.sub(r'[^A-Za-z0-9_.-]+', '_', label)
    stdout_path = out / 'logs' / f'{safe}.stdout'
    stderr_path = out / 'logs' / f'{safe}.stderr'
    stdout_path.parent.mkdir(parents=True, exist_ok=True)
    stdout_path.write_bytes(result.stdout)
    stderr_path.write_bytes(result.stderr)
    row = {'label': label, 'argv': [str(v) for v in argv], 'cwd': str(cwd),
           'exit': result.returncode, 'stdout': stdout_path.relative_to(out).as_posix(),
           'stdout_bytes': len(result.stdout), 'stdout_sha256': sha_file(stdout_path),
           'stderr': stderr_path.relative_to(out).as_posix(),
           'stderr_bytes': len(result.stderr), 'stderr_sha256': sha_file(stderr_path),
           'environment_policy': {'removed_jvm_option_variables': cleared}}
    commands.append(row)
    return result, row


def find_compile_command(manifest: dict, case: dict, dataset: str) -> dict:
    leg = case['leg']
    family = 'constructor-primitive-conversions' if dataset == 'numeric' else case['family']
    if dataset == 'numeric':
        suffix = f'{leg}-original-compile'
    elif dataset == 'fixture':
        suffix = f'fixture/{family}/{leg}/compile'
    elif family in ('ct-legacy-frozen', 'bigdecimal-control-corrected'):
        suffix = f'addendum/{family}/{leg}/compile'
    else:
        suffix = f'baseline/{family}/{leg}/compile'
    matches = [row for row in manifest['commands'] if row['label'].endswith(suffix)]
    if len(matches) != 1:
        raise SystemExit(f'cannot identify one frozen compile command for {dataset}/{family}/{leg}')
    return matches[0]


def raw_stream_capture(dataset: str, family: str, leg: str, old_case: dict,
                       old_root: pathlib.Path, numeric_manifest: dict | None) -> tuple[bytes, bytes, str, str]:
    if dataset == 'fixture':
        if family == 'direct':
            direct_root = REPO / 'tests/fixtures/p3-heterogeneous-array-initializers-v3/direct'
            stdout = direct_root / leg / 'logs' / f'direct_{leg}_original-main.stdout'
            stderr = direct_root / leg / 'logs' / f'direct_{leg}_original-main.stderr'
            return stdout.read_bytes(), stderr.read_bytes(), str(stdout), 'exact direct original capture from frozen pathfix fixture'
        source_case = old_case
        runtime = source_case['runtime']
        stdout = old_root / runtime['stdout']; stderr = old_root / runtime['stderr']
        return stdout.read_bytes(), stderr.read_bytes(), str(stdout), 'permanent factory original capture'
    if dataset == 'numeric':
        source_cmds = numeric_manifest['commands']
        suffix = f'{leg}-original-run'
        rows = [row for row in source_cmds if row['label'].endswith(suffix)]
        if len(rows) != 1:
            raise SystemExit(f'cannot identify numeric baseline raw run for {leg}')
        stdout = NUMERIC_ROOT / rows[0]['stdout']; stderr = NUMERIC_ROOT / rows[0]['stderr']
        return stdout.read_bytes(), stderr.read_bytes(), str(stdout), 'permanent numeric baseline raw capture'
    if family == 'bigdecimal-control-corrected':
        leg_root = BIGDECIMAL_ROOT / 'raw' / leg
        stdout = leg_root / 'logs' / f'{leg}_bigdecimal-corrected_original-run.stdout'
        stderr = leg_root / 'logs' / f'{leg}_bigdecimal-corrected_original-run.stderr'
        return stdout.read_bytes(), stderr.read_bytes(), str(stdout), 'permanent BigDecimal provenance archive'
    archived_family = 'ct-legacy' if family == 'ct-legacy-frozen' else family
    archive_member = f'jarde-em18-baseline-20261009/cases/{archived_family}/{leg}/logs/{archived_family}_{leg}_original-run.'
    with tarfile.open(BASELINE_ARCHIVE, 'r:gz') as archive:
        stdout_member = archive.extractfile(archive_member + 'stdout')
        stderr_member = archive.extractfile(archive_member + 'stderr')
        if stdout_member is None or stderr_member is None:
            raise SystemExit(f'historical baseline tar is missing original streams for {family}/{leg}')
        kind = ('permanent CT original capture reused for frozen-CT leg with identical frozen hashes'
                if family == 'ct-legacy-frozen' else 'permanent baseline evidence archive')
        return (stdout_member.read(), stderr_member.read(),
                f'{BASELINE_ARCHIVE}::{archive_member}stdout', kind)


def frozen_jdk_identity(audit: dict, leg: str) -> dict:
    row = next(row for row in audit['direct_javap_evidence']['legs'] if row['leg'] == leg)
    return {'jdk_home': row['jdk_home'], 'javac_sha256': row['tools_sha256']['javac'],
            'java_sha256': row['tools_sha256']['java']}


def report_facts(document: dict, source_path: pathlib.Path, out: pathlib.Path) -> dict:
    text = document['text']
    methods = []
    for method in document.get('methods', []):
        outcome = method.get('outcome', {})
        report = outcome.get('report') if outcome.get('kind') == 'recovered' else None
        body = report.get('text', '') if report else ''
        source_map = report.get('source_map', {}) if report else {}
        segments = source_map.get('segments', [])
        primary, all_bcis = set(), set()
        for segment in segments:
            origin = segment.get('origin', {})
            bci = origin.get('primary', {}).get('bci')
            if isinstance(bci, int):
                primary.add(bci); all_bcis.add(bci)
            for derived in origin.get('derived', []):
                dbci = derived.get('bci')
                if isinstance(dbci, int):
                    all_bcis.add(dbci)
        news = report.get('news', []) if report else []
        item = method.get('item', {})
        name = item.get('name', {})
        descriptor = item.get('descriptor', {})
        methods.append({
            'member_index': item.get('index'),
            'member_declaration': method.get('declaration', item.get('declaration')),
            'name': {'raw': name.get('raw'), 'escaped': name.get('escaped'),
                     'utf16': name.get('utf16')},
            'descriptor': {'raw': descriptor.get('raw'), 'escaped': descriptor.get('escaped'),
                           'utf16': descriptor.get('utf16')},
            'access_flags': item.get('access_flags'),
            'member_identity': item.get('identity'),
            'body_identity': item.get('body'),
            'outcome_kind': outcome.get('kind'), 'analysis': outcome.get('analysis'),
            'refusal': outcome.get('refusal'), 'member_markers': method.get('markers', []),
            'quality': report.get('quality') if report else None,
            'representation': report.get('representation') if report else None,
            'diagnostics': report.get('diagnostics') if report else None,
            'body_text': body if report else None,
            'body_sha256': sha_bytes(body.encode('utf-8')) if report else None,
            'body_bytecode_marker': '@bytecode' in body if report else None,
            'body_refused_marker': 'jarde_refused_body' in body if report else None,
            'new_records': [{k: rec.get(k) for k in
                             ('class', 'head', 'dup', 'constructor', 'arguments', 'presented', 'refusal')}
                            for rec in news],
            'source_map': {'sha256': canonical_hash(source_map), 'segment_count': len(segments),
                           'primary_bcis': sorted(primary), 'all_bcis': sorted(all_bcis)},
        })
    return {'class': document.get('class'), 'declaration': document.get('declaration'),
            'source': file_record(source_path, out), 'source_text_sha256': sha_bytes(text.encode()),
            'source_bytecode_marker': '@bytecode' in text,
            'source_refused_body_marker': 'jarde_refused_body' in text,
            'coverage': document.get('coverage'), 'diagnostics': document.get('diagnostics'),
            'methods': methods}


def collection_grid_signature_summary(report_rows: list[dict], baseline: dict,
                                      leg: str) -> dict:
    baseline_leg = next(row for row in baseline['legs'] if row['leg'] == leg)
    matches = []
    target_name = list('collectionGridDirect'.encode('utf-8'))
    for report_row in report_rows:
        source_report = report_row.get('source_report') or {}
        for method in source_report.get('methods', []):
            if method.get('name', {}).get('raw') == target_name:
                matches.append((report_row, source_report, method))
    if len(matches) != 1:
        return {
            'physical_descriptor': baseline_leg['physical_descriptor'],
            'physical_signature': baseline_leg['physical_signature'],
            'original_declaration': baseline_leg['declaration'],
            'candidate_method_count': len(matches),
            'candidate_method': None,
        }
    report_row, source_report, method = matches[0]
    source = source_report.get('source') or {}
    return {
        'physical_descriptor': baseline_leg['physical_descriptor'],
        'physical_signature': baseline_leg['physical_signature'],
        'original_declaration': baseline_leg['declaration'],
        'candidate_method_count': 1,
        'candidate_method': {
            'class': report_row.get('class'),
            'source_path': source.get('path'),
            'source_sha256': source.get('sha256'),
            'member_declaration': method.get('member_declaration'),
            'member_markers': method.get('member_markers'),
            'outcome_kind': method.get('outcome_kind'),
            'refusal': method.get('refusal'),
            'quality': method.get('quality'),
            'representation': method.get('representation'),
            'body_text': method.get('body_text'),
            'body_sha256': method.get('body_sha256'),
            'body_bytecode_marker': method.get('body_bytecode_marker'),
            'body_refused_marker': method.get('body_refused_marker'),
            'new_records': method.get('new_records'),
            'source_map': method.get('source_map'),
        },
    }


def main() -> int:
    if len(sys.argv) != 4:
        raise SystemExit('usage: replay-candidate-root-v1.py FROZEN_CLI CLI_METADATA_JSON ABSOLUTE_OUTPUT_DIR')
    cli = pathlib.Path(sys.argv[1]).resolve()
    metadata_path = pathlib.Path(sys.argv[2]).resolve()
    raw_out = pathlib.Path(sys.argv[3])
    if not raw_out.is_absolute():
        raise SystemExit(f'output directory must be absolute: {raw_out}')
    out = raw_out.resolve()
    if not cli.is_file() or not metadata_path.is_file():
        raise SystemExit('candidate CLI and metadata must both exist')
    if not out.is_absolute() or out.exists():
        raise SystemExit(f'refusing non-absolute or existing output directory: {out}')
    cli_sha = sha_file(cli)
    metadata = json.loads(metadata_path.read_text(encoding='utf-8'))
    if pathlib.Path(metadata.get('cli_path', '')).resolve() != cli or metadata.get('cli_sha256') != cli_sha:
        raise SystemExit('candidate CLI binary does not match the supplied frozen metadata identity')
    candidate_source_hashes = metadata.get('candidate_sources', {})
    if not isinstance(candidate_source_hashes, dict) or set(candidate_source_hashes) != REQUIRED_CANDIDATE_SOURCES:
        raise SystemExit(f'candidate metadata must pin exactly these source identities: {sorted(REQUIRED_CANDIDATE_SOURCES)}')
    for relative, expected in candidate_source_hashes.items():
        source = REPO / relative
        if not source.is_file() or sha_file(source) != expected:
            raise SystemExit(f'candidate source identity mismatch: {source}')

    family = json.loads(FAMILY_MANIFEST_PATH.read_text(encoding='utf-8'))
    matrix = json.loads(MATRIX_MANIFEST_PATH.read_text(encoding='utf-8'))
    numeric = json.loads(NUMERIC_MANIFEST_PATH.read_text(encoding='utf-8'))
    bigdecimal = json.loads(BIGDECIMAL_MANIFEST_PATH.read_text(encoding='utf-8'))
    baseline_audit = json.loads(BASELINE_REFERENCE_AUDIT_PATH.read_text(encoding='utf-8'))
    baseline_verification = json.loads(BASELINE_ROOT_VERIFICATION_PATH.read_text(encoding='utf-8'))
    generic_baseline = json.loads(GENERIC_BASELINE_SUMMARY_PATH.read_text(encoding='utf-8'))
    if (generic_baseline.get('verification') != 'baseline_frozen_signature_pending'
            or generic_baseline.get('fresh_execution') is not False):
        raise SystemExit('six-class candidate baseline summary is not the expected historical pending-Signature record')
    generic_candidate_manifest_path = pathlib.Path(generic_baseline['manifest_path'])
    if (not generic_candidate_manifest_path.is_file()
            or sha_file(generic_candidate_manifest_path) != generic_baseline['manifest_sha256']):
        raise SystemExit('six-class baseline candidate manifest is missing or changed')
    generic_root_verification_path = pathlib.Path(generic_baseline['root_verification_path'])
    if (not generic_root_verification_path.is_file()
            or sha_file(generic_root_verification_path) != generic_baseline['root_verification_sha256']):
        raise SystemExit('six-class baseline root verification is missing or changed')
    if generic_baseline.get('candidate_cli_sha256') != '27e53f610f1915c16c21fb144d63aebafb7d376c3eff349b6b5de7673a313123':
        raise SystemExit('six-class baseline summary refers to an unexpected historical candidate CLI')
    if baseline_verification.get('verification') != 'passed' or baseline_verification.get('errors'):
        raise SystemExit('six-class baseline root verification is not clean')
    if baseline_verification.get('audit_sha256') != sha_file(BASELINE_REFERENCE_AUDIT_PATH):
        raise SystemExit('six-class baseline audit does not match its root verification')
    archive_metadata = json.loads(BASELINE_ARCHIVE_METADATA.read_text(encoding='utf-8'))
    if sha_file(BASELINE_ARCHIVE) != archive_metadata['archive_sha256']:
        raise SystemExit('permanent EM-18 baseline archive hash mismatch')
    with tarfile.open(BASELINE_ARCHIVE, 'r:gz') as archive:
        manifest_member = archive.extractfile('jarde-em18-baseline-20261009/manifest.json')
        if manifest_member is None or sha_bytes(manifest_member.read()) != archive_metadata['primary_manifest_sha256']:
            raise SystemExit('permanent baseline archive primary manifest identity mismatch')
    verify_frozen_files(FAMILY_ROOT, family, 'legacy-family-cli2-root-v1')
    verify_frozen_files(MATRIX_ROOT, matrix, 'legacy-18-cli2-root-v1')
    verify_frozen_files(NUMERIC_ROOT, numeric, 'numeric baseline-v1')
    verify_frozen_files(BIGDECIMAL_ROOT, bigdecimal, 'BigDecimal provenance archive')
    direct_identity = {'root': baseline_audit['current_frozen_direct']['root'],
                       'class_names': baseline_audit['current_frozen_direct']['class_names'],
                       'sources': [], 'legs': {}}
    for source in baseline_audit['current_frozen_direct']['source_hashes']:
        source_path = pathlib.Path(source['path'])
        if not source_path.is_file() or source_path.stat().st_size != source['bytes'] or sha_file(source_path) != source['sha256']:
            raise SystemExit(f'frozen six-class direct source changed: {source_path}')
        direct_identity['sources'].append({'path': str(source_path), 'bytes': source['bytes'],
                                           'sha256': source['sha256']})
    if {pathlib.Path(row['path']).stem for row in direct_identity['sources']} != set(direct_identity['class_names']):
        raise SystemExit('frozen direct source closure does not match the six-class set')
    for leg in ('javac8', 'javac23'):
        tool_identity = frozen_jdk_identity(baseline_audit, leg)
        class_records = []
        for class_row in baseline_audit['current_frozen_direct']['class_hashes_by_jdk'][leg]:
            class_path = pathlib.Path(class_row['path'])
            if not class_path.is_file() or class_path.stat().st_size != class_row['bytes'] or sha_file(class_path) != class_row['sha256']:
                raise SystemExit(f'frozen six-class {leg} class changed: {class_path}')
            class_records.append({'path': str(class_path), 'bytes': class_row['bytes'],
                                  'sha256': class_row['sha256']})
        if {pathlib.Path(row['path']).stem for row in class_records} != set(direct_identity['class_names']):
            raise SystemExit(f'frozen direct {leg} class closure does not match the six-class set')
        for tool in ('javac', 'java'):
            binary = pathlib.Path(tool_identity['jdk_home']) / 'bin' / tool
            if not binary.is_file() or sha_file(binary) != tool_identity[f'{tool}_sha256']:
                raise SystemExit(f'frozen six-class {leg} JDK tool identity changed: {binary}')
        direct_identity['legs'][leg] = {'jdk_home': tool_identity['jdk_home'],
                                        'tool_sha256': {'javac': tool_identity['javac_sha256'],
                                                        'java': tool_identity['java_sha256']},
                                        'class_files': class_records}
    commands: list[dict] = []
    cases: list[dict] = []
    jobs = []
    for row in family['cases']:
        jobs.append(('fixture', row['family'], FAMILY_ROOT, family, row))
    for row in matrix['cases']:
        jobs.append(('baseline', row['family'], MATRIX_ROOT, matrix, row))
    numeric_cases = {row['leg']: row for row in numeric['cases'] if row['profile'] == 'jarde-cli2'}
    for leg, row in sorted(numeric_cases.items()):
        jobs.append(('numeric', 'constructor-primitive-conversion', NUMERIC_ROOT, numeric, row))
    if len(jobs) != 24:
        raise SystemExit(f'expected 22 legacy + 2 numeric candidate legs; got {len(jobs)}')

    # Validate the entire denominator and every immutable original byte stream before
    # starting any candidate work; a missing historical capture must not make a partial
    # matrix look complete.
    for dataset, family_name, old_root, old_manifest, old_case in jobs:
        leg = old_case['leg']
        if dataset == 'numeric':
            old_jar = NUMERIC_ROOT / old_case['input_jar']['path']
            expected_jar_sha = old_case['input_jar']['sha256']
        else:
            old_jar = pathlib.Path(old_case['input_jar'])
            expected_jar_sha = old_case['input_jar_sha256']
            if dataset == 'baseline' and family_name == 'bigdecimal-control-corrected':
                old_jar = BIGDECIMAL_ROOT / 'raw' / leg / 'input.jar'
        if not old_jar.is_file() or sha_file(old_jar) != expected_jar_sha:
            raise SystemExit(f'preflight {dataset}/{family_name}/{leg}: missing or changed input JAR {old_jar}')
        compile_old = find_compile_command(old_manifest, old_case, dataset)
        javac = pathlib.Path(compile_old['argv'][0])
        java = javac.parent / 'java'
        expected_tools = frozen_jdk_identity(baseline_audit, leg)
        expected_home = pathlib.Path(expected_tools['jdk_home'])
        if (javac != expected_home / 'bin' / 'javac'
                or java != expected_home / 'bin' / 'java'
                or not javac.is_file() or not java.is_file()
                or sha_file(javac) != expected_tools['javac_sha256']
                or sha_file(java) != expected_tools['java_sha256']):
            raise SystemExit(f'preflight {dataset}/{family_name}/{leg}: frozen JDK executable identity changed')
        stdout_bytes, stderr_bytes, _raw_path, _raw_kind = raw_stream_capture(
            dataset, family_name, leg, old_case, old_root, numeric if dataset == 'numeric' else None)
        original = old_case.get('original_run') if dataset != 'numeric' else None
        if dataset == 'numeric':
            run_rows = [row for row in numeric['commands'] if row['label'].endswith(f'{leg}-original-run')]
            if len(run_rows) != 1:
                raise SystemExit(f'preflight numeric/{leg}: original run command is ambiguous')
            original = {'stdout_sha256': run_rows[0]['stdout_sha256'],
                        'stderr_sha256': run_rows[0]['stderr_sha256']}
        for raw_bytes, key in ((stdout_bytes, 'stdout_sha256'), (stderr_bytes, 'stderr_sha256')):
            if sha_bytes(raw_bytes) != original[key]:
                raise SystemExit(f'preflight {dataset}/{family_name}/{leg}: historical {key} raw missing or changed')

    out.mkdir(parents=True)
    (out / 'logs').mkdir()
    (out / 'inputs').mkdir()

    for index, (dataset, family_name, old_root, old_manifest, old_case) in enumerate(jobs):
        leg = old_case['leg']
        if dataset == 'numeric':
            old_jar = NUMERIC_ROOT / old_case['input_jar']['path']
            expected_jar_sha = old_case['input_jar']['sha256']
        else:
            old_jar = pathlib.Path(old_case['input_jar'])
            expected_jar_sha = old_case['input_jar_sha256']
        if dataset == 'baseline' and family_name == 'bigdecimal-control-corrected':
            old_jar = BIGDECIMAL_ROOT / 'raw' / leg / 'input.jar'
        if not old_jar.is_file() or sha_file(old_jar) != expected_jar_sha:
            raise SystemExit(f'{dataset}/{family_name}/{leg}: frozen input jar identity mismatch: {old_jar}')
        copied_jar = out / 'inputs' / f'{index:02d}-{dataset}-{family_name}-{leg}.jar'
        shutil.copyfile(old_jar, copied_jar)
        if sha_file(copied_jar) != expected_jar_sha:
            raise SystemExit(f'copied input jar hash mismatch: {copied_jar}')
        entries = class_entries(copied_jar)
        class_names = [row['entry'][:-6].replace('/', '.') for row in entries]
        if dataset in ('fixture', 'baseline'):
            expected_report_classes = sorted(row['class'] for row in old_case['reports'])
            if sorted(class_names) != expected_report_classes:
                raise SystemExit(f'{dataset}/{family_name}/{leg}: full jar class closure differs from frozen report set')
        elif dataset == 'numeric':
            expected_entries = sorted(row['name'] for row in old_case['input_class_set'])
            if sorted(row['entry'].split('/')[-1] for row in entries) != expected_entries:
                raise SystemExit(f'{dataset}/{leg}: numeric jar class closure differs from frozen manifest')
        if dataset == 'numeric':
            main_class = old_case['resolved_main_class']
            if main_class not in class_names:
                raise SystemExit(f'{dataset}/{leg}: frozen numeric main class is absent from full input class set')
        else:
            mains = [name for name in class_names if name.rsplit('.', 1)[-1] in ('Main', 'CT')]
            if not mains:
                mains = [name for name in class_names if name.rsplit('.', 1)[-1] == 'Base']
            if len(mains) != 1:
                raise SystemExit(f'{dataset}/{family_name}/{leg}: expected exactly one Main/CT/Base class: {mains}')
            main_class = mains[0]

        old_compile = find_compile_command(old_manifest, old_case, dataset)
        javac = pathlib.Path(old_compile['argv'][0])
        java = javac.parent / 'java'
        cp_i = old_compile['argv'].index('-classpath')
        compiler_flags = old_compile['argv'][1:cp_i]
        case_root = out / dataset / family_name / leg
        sources, classes, empty = case_root / 'sources', case_root / 'classes', case_root / 'empty-classpath-sourcepath'
        for directory in (sources, classes, empty):
            directory.mkdir(parents=True)
        prefix = f'{dataset}/{family_name}/{leg}'
        report_rows, source_paths = [], []
        for class_name in class_names:
            rendered, render_cmd = command(
                f'{prefix}/render-{class_name}',
                [str(cli), 'class-source', '--input', str(copied_jar), '--class', class_name,
                 '--policy', 'plain-jar', '--format', 'json', '--evidence', 'all', '--release', '8'],
                WORKSPACE, out, commands)
            row = {'class': class_name, 'command': render_cmd, 'json_parsed': False,
                   'source_report': None, 'parse_error': None}
            if rendered.returncode == 0:
                try:
                    doc = json.loads(rendered.stdout)
                    java_source = doc['text']
                    relative = pathlib.Path(*class_name.split('.')).with_suffix('.java')
                    source_path = sources / relative
                    source_path.parent.mkdir(parents=True, exist_ok=True)
                    source_path.write_text(java_source, encoding='utf-8')
                    source_paths.append(source_path)
                    row['json_parsed'] = True
                    row['source_report'] = report_facts(doc, source_path, out)
                except (json.JSONDecodeError, KeyError, TypeError, OSError) as error:
                    row['parse_error'] = str(error)
            report_rows.append(row)
        expected_paths = {pathlib.Path(*name.split('.')).with_suffix('.java').as_posix() for name in class_names}
        actual_paths = {path.relative_to(sources).as_posix() for path in source_paths}
        source_set_complete = len(source_paths) == len(class_names) and actual_paths == expected_paths
        no_stubs = source_set_complete and all(
            row['json_parsed'] and not row['source_report']['source_bytecode_marker']
            and not row['source_report']['source_refused_body_marker'] for row in report_rows)
        compile_result, compile_record = command(
            f'{prefix}/compile-all-candidate-sources',
            [str(javac), *compiler_flags, '-classpath', str(empty), '-sourcepath', str(empty),
             '-d', str(classes), *[str(p) for p in sorted(source_paths)]],
            case_root, out, commands)
        candidate_class_files = sorted(classes.rglob('*.class'))
        candidate_class_set = [p.relative_to(classes).as_posix() for p in candidate_class_files]

        if dataset != 'numeric':
            original = old_case['original_run']
        else:
            numeric_run_rows = [row for row in numeric['commands']
                                if row['label'].endswith(f'{leg}-original-run')]
            if len(numeric_run_rows) != 1:
                raise SystemExit(f'{prefix}: cannot identify numeric original run command')
            original_cmd = numeric_run_rows[0]
            original = {'exit': original_cmd['exit'],
                        'stdout_sha256': original_cmd['stdout_sha256'],
                        'stderr_sha256': original_cmd['stderr_sha256']}
        original_stdout, original_stderr, original_source, original_kind = raw_stream_capture(
            dataset, family_name, leg, old_case, old_root, numeric if dataset == 'numeric' else None)
        for stream_path, expected_hash, name in (
            (original_stdout, original['stdout_sha256'], 'stdout'),
            (original_stderr, original['stderr_sha256'], 'stderr')):
            if sha_bytes(stream_path) != expected_hash:
                raise SystemExit(f'{prefix}: original {name} raw stream disagrees with its frozen hash')
        original_copy_dir = out / 'original-streams'
        original_copy_dir.mkdir(exist_ok=True)
        copied_stdout = original_copy_dir / f'{index:02d}-{dataset}-{family_name}-{leg}.stdout'
        copied_stderr = original_copy_dir / f'{index:02d}-{dataset}-{family_name}-{leg}.stderr'
        copied_stdout.write_bytes(original_stdout); copied_stderr.write_bytes(original_stderr)

        runtime_record = None
        if compile_result.returncode == 0 and source_set_complete:
            _, runtime_record = command(f'{prefix}/run-new-classes',
                                        [str(java), '-Xverify:all', '-cp', str(classes), main_class],
                                        case_root, out, commands)
        stream_match = bool(runtime_record and
                            (out / runtime_record['stdout']).read_bytes() == copied_stdout.read_bytes() and
                            (out / runtime_record['stderr']).read_bytes() == copied_stderr.read_bytes())
        exit_match = bool(runtime_record and runtime_record['exit'] == original['exit'])
        cases.append({
            'dataset': dataset, 'family': family_name, 'leg': leg,
            'historical_candidate_result': {
                'manifest_path': str(NUMERIC_MANIFEST_PATH if dataset == 'numeric'
                                     else FAMILY_MANIFEST_PATH if dataset == 'fixture'
                                     else MATRIX_MANIFEST_PATH),
                'accepted': old_case.get('accepted'),
                'compile_exit': old_case.get('compile_exit', old_case.get('compile', {}).get('exit')),
                'report_markers': ([{'class': row.get('class'),
                                     'bytecode_marker': row.get('bytecode_marker'),
                                     'refused_body_marker': row.get('refused_body_marker')}
                                    for row in old_case.get('reports', [])]
                                   if dataset != 'numeric' else
                                   [{'class': row.get('class'),
                                     'bytecode_marker': row.get('bytecode_marker'),
                                     'refused_body_marker': row.get('refused_body_marker')}
                                    for row in old_case.get('reports', [])])},
            'input_jar': {'source_path': str(old_jar), 'sha256': expected_jar_sha,
                          'copied_path': copied_jar.relative_to(out).as_posix(),
                          'class_entries': entries},
            'main_class': main_class, 'javac': str(javac), 'java': str(java),
            'tool_identity': {'javac_sha256': sha_file(javac), 'java_sha256': sha_file(java)},
            'compiler_flags_reused_from_frozen_command': compiler_flags,
            'frozen_original_raw_streams': {
                'source': original_source, 'capture_kind': original_kind, 'fresh': False,
                'stdout_copy': copied_stdout.relative_to(out).as_posix(),
                'stderr_copy': copied_stderr.relative_to(out).as_posix(),
                'exit': original['exit'], 'stdout_sha256': original['stdout_sha256'],
                'stderr_sha256': original['stderr_sha256']},
            'reports': report_rows, 'source_set_complete': source_set_complete,
            'collection_grid_signature_target': (
                collection_grid_signature_summary(report_rows, generic_baseline, leg)
                if dataset == 'fixture' and family_name == 'direct' else None),
            'all_sources_without_bytecode_or_refused_body_markers': no_stubs,
            'candidate_compile': compile_record,
            'candidate_class_set': candidate_class_set,
            'candidate_runtime': runtime_record,
            'runtime_raw_streams_match_original_bytes': stream_match,
            'runtime_exit_matches_original': exit_match,
            'candidate_success': bool(source_set_complete and no_stubs and compile_result.returncode == 0
                                      and runtime_record and runtime_record['exit'] == 0
                                      and stream_match and exit_match),
        })
        (out / 'manifest.partial.json').write_text(
            json.dumps({'schema': 'recover-reifiable-wildcard-array-return-signatures-candidate-root-v1',
                        'runner_sha256': sha_file(pathlib.Path(__file__).resolve()),
                        'cli_sha256': cli_sha, 'completed_cases': cases, 'commands': commands},
                       indent=2, ensure_ascii=False) + '\n', encoding='utf-8')

    historical_jadx_direct = [row for row in baseline_audit['old_jadx']['verification_summary']['runs']
                              if row['key'].startswith('direct/')]
    historical_jarde_direct = baseline_audit['old_heterogeneous_jarde']['replay_v2_direct_cases']
    manifest = {
        'schema': 'recover-reifiable-wildcard-array-return-signatures-candidate-root-v1',
        'runner_path': str(pathlib.Path(__file__).resolve()),
        'runner_sha256': sha_file(pathlib.Path(__file__).resolve()),
        'candidate_cli': {'path': str(cli), 'sha256': cli_sha,
                          'metadata_path': str(metadata_path), 'metadata_sha256': sha_file(metadata_path),
                          'metadata': metadata},
        'historical_evidence': {
            'legacy_family_manifest': {'path': str(FAMILY_MANIFEST_PATH), 'sha256': sha_file(FAMILY_MANIFEST_PATH)},
            'legacy_18_manifest': {'path': str(MATRIX_MANIFEST_PATH), 'sha256': sha_file(MATRIX_MANIFEST_PATH)},
            'numeric_baseline_manifest': {'path': str(NUMERIC_MANIFEST_PATH), 'sha256': sha_file(NUMERIC_MANIFEST_PATH)},
            'bigdecimal_archive_manifest': {'path': str(BIGDECIMAL_MANIFEST_PATH), 'sha256': sha_file(BIGDECIMAL_MANIFEST_PATH)},
            'six_class_baseline_reference_audit': {
                'path': str(BASELINE_REFERENCE_AUDIT_PATH),
                'sha256': sha_file(BASELINE_REFERENCE_AUDIT_PATH),
                'root_verification_path': str(BASELINE_ROOT_VERIFICATION_PATH),
                'root_verification_sha256': sha_file(BASELINE_ROOT_VERIFICATION_PATH),
                'root_verification_checks': baseline_verification['checks'],
                'six_class_direct_identity': direct_identity,
                'historical_jarde_direct_legs': historical_jarde_direct,
                'historical_jadx_direct_profiles': historical_jadx_direct,
                'historical_only_not_fresh_candidate': True},
            'wildcard_signature_pending_baseline': {
                'path': str(GENERIC_BASELINE_SUMMARY_PATH),
                'sha256': sha_file(GENERIC_BASELINE_SUMMARY_PATH),
                'candidate_manifest_path': str(generic_candidate_manifest_path),
                'candidate_manifest_sha256': sha_file(generic_candidate_manifest_path),
                'root_verification_path': str(generic_root_verification_path),
                'root_verification_sha256': sha_file(generic_root_verification_path),
                'fresh_execution': False,
                'collection_grid_direct_original_signature_by_leg': [
                    {'leg': row['leg'], 'physical_descriptor': row['physical_descriptor'],
                     'physical_signature': row['physical_signature'],
                     'declaration': row['declaration'], 'body_sha256': row['body_sha256'],
                     'markers': row['markers']} for row in generic_baseline['legs']]},
            'permanent_em18_baseline_archive': {'path': str(BASELINE_ARCHIVE),
                                                'sha256': sha_file(BASELINE_ARCHIVE),
                                                'metadata_path': str(BASELINE_ARCHIVE_METADATA),
                                                'metadata_sha256': sha_file(BASELINE_ARCHIVE_METADATA),
                                                'primary_manifest_sha256': archive_metadata['primary_manifest_sha256']},
            'prior_candidate_results_are_not_reused_as_fresh_candidate_output': True,
        },
        'runtime_environment_policy': 'Each subprocess explicitly removes JAVA_TOOL_OPTIONS, _JAVA_OPTIONS, JDK_JAVA_OPTIONS; whether each was inherited is recorded per command.',
        'input_matrix': {'legacy_family_cases': len(family['cases']), 'legacy_18_cases': len(matrix['cases']),
                         'numeric_cases': len(numeric_cases), 'total_fresh_candidate_legs': len(cases)},
        'historical_denominators': {'factory': '2/2 accepted', 'legacy_18_matrix': '16/18 accepted',
                                    'direct_six_class_family': '0/2 accepted historically',
                                    'bigdecimal': '2 known failures included within legacy 18; not omitted'},
        'candidate_success_metric': (
            'complete source set, no bytecode/refused-body stubs, compile and runtime exit 0, '
            'and exact original stdout/stderr plus exit equality; this does not claim generic '
            'Signature projection success'),
        'candidate_cases': cases, 'commands': commands,
        'candidate_success_count': sum(bool(row['candidate_success']) for row in cases),
        'status': 'complete_replay_recorded' if len(cases) == 24 else 'incomplete',
    }
    (out / 'manifest.partial.json').unlink()
    manifest['files'] = [file_record(path, out) for path in sorted(out.rglob('*'))
                         if path.is_file() and path.name not in ('manifest.json', 'manifest.partial.json')]
    (out / 'manifest.json').write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + '\n', encoding='utf-8')
    print(json.dumps({'manifest': str(out / 'manifest.json'), 'runner_sha256': manifest['runner_sha256'],
                      'cli_sha256': cli_sha, 'legs': len(cases),
                      'candidate_successes': manifest['candidate_success_count']}, indent=2))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
