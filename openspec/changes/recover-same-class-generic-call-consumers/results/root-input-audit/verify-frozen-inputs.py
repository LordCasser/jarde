#!/usr/bin/env python3
"""独立核对冻结输入，不编译、不运行目标、不改写冻结证据。"""
import hashlib
import json
from collections import Counter
from pathlib import Path
from zipfile import ZipFile


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


repo = Path(__file__).resolve().parents[5]
root = repo / 'openspec/evidence/same-class-generic-call-consumers-2026-10-09/frozen-inputs-v1'
manifest_path = root / 'frozen-input-manifest.json'
manifest = json.loads(manifest_path.read_text())
errors = []
checked = 0
for record in manifest['files']:
    path = root / record['path']
    if not path.is_file() or digest(path) != record['sha256'] or path.stat().st_size != record['bytes']:
        errors.append({'kind': 'file-mismatch', 'path': record['path']})
    checked += 1
keys = [(r['name'], r['leg'], r['debug']) for r in manifest['inputs']]
if len(keys) != 140 or len(set(keys)) != 140:
    errors.append({'kind': 'input-count-or-duplicate', 'count': len(keys), 'unique': len(set(keys))})
family_legs = {}
class_count = 0
for row in manifest['inputs']:
    family_legs.setdefault(row['name'], set()).add((row['leg'], row['debug']))
    jar = root / row['jar']
    if digest(jar) != row['jar_sha256']:
        errors.append({'kind': 'jar-mismatch', 'input': row['jar']})
    for source, expected in zip(row['source'], row['source_sha256']):
        if digest(root / source) != expected:
            errors.append({'kind': 'source-mismatch', 'input': source})
    original = repo / row['source_original_path']
    if len(row['source']) != 1 or digest(original) != row['source_sha256'][0]:
        errors.append({'kind': 'source-original-mismatch', 'input': row['jar']})
    if row['kind'] == 'reused-existing-jar' and digest(repo / row['jar_original_path']) != row['jar_sha256']:
        errors.append({'kind': 'historical-jar-mismatch', 'input': row['jar']})
    with ZipFile(jar) as archive:
        names = [n for n in archive.namelist() if n.endswith('.class')]
        class_count += len(names)
        if not names or len(names) != len(set(names)):
            errors.append({'kind': 'empty-or-duplicate-jar-classes', 'input': row['jar']})
        expected_classes = row.get('class_sha256')
        if expected_classes is not None:
            actual_classes = {n: hashlib.sha256(archive.read(n)).hexdigest() for n in names}
            if actual_classes != expected_classes:
                errors.append({'kind': 'jar-class-bytes-mismatch', 'input': row['jar']})
    if row['javap_exit'] != 0:
        errors.append({'kind': 'input-javap-failed', 'input': row['jar']})
expected_legs = {(jdk, debug) for jdk in ('corretto8', 'openjdk23') for debug in ('debug', 'nodebug')}
if len(family_legs) != 35 or any(legs != expected_legs for legs in family_legs.values()):
    errors.append({'kind': 'missing-family-leg'})
compile_commands = [c for c in manifest['commands'] if c['label'] == 'freeze-javac']
if len(compile_commands) != 128:
    errors.append({'kind': 'new-input-compile-count', 'count': len(compile_commands)})
for command in manifest['commands']:
    if command['returncode'] != 0:
        errors.append({'kind': 'input-command-failed', 'label': command['label'], 'cwd': command['cwd']})
    for stream in ('stdout', 'stderr'):
        path = Path(command['cwd']) / (command['label'] + '.' + stream)
        if not path.is_file() or digest(path) != command[stream + '_sha256']:
            errors.append({'kind': 'input-command-output-mismatch', 'path': str(path)})
for command in compile_commands:
    args = command['argv']
    for flag in ('-classpath', '-sourcepath'):
        value = Path(args[args.index(flag) + 1])
        if not value.is_dir() or any(value.iterdir()):
            errors.append({'kind': 'nonempty-input-compile-search-path', 'path': str(value)})
loose_classes = [str(p.relative_to(root)) for p in root.rglob('*.class')]
if loose_classes:
    errors.append({'kind': 'unremoved-generated-classes', 'paths': loose_classes})
result = {
    'scope': '冻结输入身份、四腿完整性、真实输入编译命令和结果；不代表反编译候选验收',
    'manifest_sha256': digest(manifest_path),
    'verifier_sha256': digest(Path(__file__)),
    'frozen_files_checked': checked,
    'unique_inputs': len(set(keys)),
    'source_families': len(family_legs),
    'input_kinds': dict(Counter(r['kind'] for r in manifest['inputs'])),
    'new_source_compile_commands': len(compile_commands),
    'jar_class_entries_checked': class_count,
    'errors': errors,
    'passed': not errors,
}
Path(__file__).with_name('verification.json').write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n')
print(json.dumps(result, ensure_ascii=False, indent=2))
raise SystemExit(bool(errors))
