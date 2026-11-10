#!/usr/bin/env python3
"""Compile the 24 existing S0 recovery sources with only their own class family filtered.

This is a preparation/measurement script. Importing the existing baseline helpers is
side-effect free; executing this script runs javac8 and javac23 only after explicit
human invocation by the root agent. It never invokes the product CLI or target code.
"""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import shutil
import sys
import zipfile

TOOL_HINT = Path(os.environ.get(
    'S0_BASELINE_TOOL',
    str(Path(__file__).with_name('s0_baseline.py'))))
TOOL = TOOL_HINT.resolve()
ROOT = TOOL.parents[4]
if not TOOL.is_file():
    raise SystemExit(f'cannot locate baseline tool: {TOOL}')
spec = importlib.util.spec_from_file_location('s0_baseline_helpers', TOOL)
base = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = base
spec.loader.exec_module(base)

CHANGE = ROOT / 'openspec/changes/restructure-demand-driven-recovery'
INPUT = CHANGE / 'results/s0-baseline-v1/real-class-results.json'
WAR_CP = CHANGE / 'results/s0-baseline-v1/real-classes/war-classpath.json'
OUT = CHANGE / 'results/s0-resumed-v2/real-class-isolated-compile'


def sha256(path):
    h = hashlib.sha256()
    with Path(path).open('rb') as f:
        for chunk in iter(lambda: f.read(1 << 20), b''):
            h.update(chunk)
    return h.hexdigest()


def file_pin(path):
    p = Path(path)
    return {'path': str(p), 'bytes': p.stat().st_size, 'sha256': sha256(p)}


def class_family_match(path, internal_name):
    """Match exactly X.class or X$*.class, with slash-separated internal name."""
    name = path.replace('\\', '/')
    if not name.endswith('.class'):
        return False
    stem = name[:-6]
    return stem == internal_name or stem.startswith(internal_name + '$')


def tree_files(root):
    root = Path(root)
    return sorted((p for p in root.rglob('*') if p.is_file()), key=lambda p: p.relative_to(root).as_posix())


def tree_pin(root):
    root = Path(root)
    return {'path': str(root), 'files': [file_pin(p) for p in tree_files(root)]}


def verify_pin(pin):
    p = Path(pin['path'])
    actual = file_pin(p)
    if actual['bytes'] != pin['bytes'] or actual['sha256'] != pin['sha256']:
        raise RuntimeError(f'pin mismatch: {p}')
    return actual


def filter_jar(source, destination, internal_name):
    """Copy all non-target logical entries with their ZipInfo; omit target family."""
    removed = []
    destination.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(source, 'r') as zin, zipfile.ZipFile(destination, 'w') as zout:
        zout.comment = zin.comment
        for ordinal, info in enumerate(zin.infolist()):
            if class_family_match(info.filename, internal_name):
                removed.append({'ordinal': ordinal, 'name': info.filename,
                                'crc': info.CRC, 'file_size': info.file_size})
                continue
            # Reuse metadata/order and preserve entry contents. Only matching family
            # entries are omitted; zipfile may recompress retained entries.
            zout.writestr(info, zin.read(info))
    return removed


def filter_directory(source, destination, internal_name):
    source, destination = Path(source), Path(destination)
    removed = []
    for p in tree_files(source):
        rel = p.relative_to(source).as_posix()
        if class_family_match(rel, internal_name):
            removed.append({'relative_path': rel, **file_pin(p)})
    if removed:
        shutil.copytree(source, destination)
        for item in removed:
            (destination / item['relative_path']).unlink()
    return removed


def source_reference_audit(rows):
    """Conservative textual token scan, not Java name resolution or dependency proof."""
    target_rows = [(r['artifact'], r['name'], r['name'].rsplit('/', 1)[-1]) for r in rows]
    audit = []
    for row in rows:
        src = Path(row['source']['path']).read_text(errors='replace')
        own = row['name']
        refs = []
        for artifact, name, simple in target_rows:
            if name == own:
                continue
            hits = [m.start() for m in re.finditer(r'(?<![\w$])' + re.escape(simple) + r'(?![\w$])', src)]
            if hits:
                refs.append({'artifact': artifact, 'internal_name': name,
                             'simple_token': simple, 'textual_occurrences': len(hits)})
        audit.append({'artifact': row['artifact'], 'internal_name': own,
                      'source_path': row['source']['path'],
                      'other_sample_target_family_tokens': refs,
                      'scope_limit': 'textual token scan only; comments/strings can match and generated/qualified/reflective references may be missed; no Java semantic resolution'})
    return audit


def make_cp_for_target(row, war_cp):
    target = row['name']
    sample_dir = OUT / row['artifact'] / f"{row['_ordinal']:02d}"
    sample_dir.mkdir(parents=True)
    removed = []
    entries = []
    if row['artifact'] == 'bcprov':
        source_jar = Path(base.ARTIFACTS[0][1])
        orig_pin = base.pin(source_jar)
        filtered_path = sample_dir / 'classpath-filtered.jar'
        with zipfile.ZipFile(source_jar) as zf:
            matches = [i for i in zf.infolist() if class_family_match(i.filename, target)]
        if matches:
            removed = filter_jar(source_jar, filtered_path, target)
            cp_path = filtered_path
            filtered_pin = base.pin(filtered_path)
        else:
            cp_path = source_jar
            filtered_pin = None
        entries.append({'kind': 'jar', 'classpath_index': 0, 'original': orig_pin,
                        'filtered': filtered_pin, 'used_path': str(cp_path),
                        'removed_target_family_entries': removed})
    else:
        class_root = Path(war_cp['classpath'].split(os.pathsep)[0])
        class_dest = sample_dir / 'classpath-classes'
        original_class_pins = war_cp['classes']
        for pin in original_class_pins:
            verify_pin(pin)
        actual_classes = sorted(str(p) for p in tree_files(class_root))
        declared_classes = sorted(p['path'] for p in original_class_pins)
        if actual_classes != declared_classes:
            raise RuntimeError('WAR classes directory differs from declared class-file pin set')
        dir_removed = filter_directory(class_root, class_dest, target)
        if dir_removed:
            used_classes = class_dest
            classes_filtered = tree_pin(class_dest)
        else:
            used_classes = class_root
            classes_filtered = None
        entries.append({'kind': 'directory', 'classpath_index': 0,
                        'original': {'path': str(class_root), 'files': original_class_pins},
                        'filtered': classes_filtered, 'used_path': str(used_classes),
                        'removed_target_family_entries': dir_removed})
        for index, jar_pin in enumerate(war_cp['jars'], start=1):
            original = verify_pin(jar_pin)
            src = Path(jar_pin['path'])
            with zipfile.ZipFile(src) as zf:
                has_match = any(class_family_match(i.filename, target) for i in zf.infolist())
            if has_match:
                filtered_path = sample_dir / f'classpath-filtered-{index:02d}.jar'
                jar_removed = filter_jar(src, filtered_path, target)
                used = filtered_path
                filtered = file_pin(filtered_path)
            else:
                jar_removed, used, filtered = [], src, None
            entries.append({'kind': 'jar', 'classpath_index': index,
                            'original': original, 'filtered': filtered,
                            'used_path': str(used),
                            'removed_target_family_entries': jar_removed})
    cp = os.pathsep.join(entry['used_path'] for entry in entries)
    (sample_dir / 'classpath.json').write_text(json.dumps({
        'artifact': row['artifact'], 'target_internal_name': target,
        'target_family_rule': 'exact internal_name.class or internal_name$*.class, across every classpath entry',
        'entry_order_preserved': True, 'entries': entries, 'classpath': cp,
        'semantic_validation': 'not_attempted'}, ensure_ascii=False, indent=2) + '\n')
    return sample_dir, cp, entries


def main():
    if OUT.exists():
        raise SystemExit(f'refusing to overwrite existing output: {OUT}')
    rows = json.loads(INPUT.read_text())
    if len(rows) != 24:
        raise SystemExit(f'expected exactly 24 existing sources; found {len(rows)}')
    for row in rows:
        verify_pin(row['source'])
    war_cp = json.loads(WAR_CP.read_text())
    # Verify each of the declared 52 jars and 119 class files before creating outputs.
    if len(war_cp['jars']) != 52:
        raise SystemExit(f"expected 52 declared WAR classpath jars; found {len(war_cp['jars'])}")
    for p in war_cp['jars']:
        verify_pin(p)
    for p in war_cp['classes']:
        verify_pin(p)
    OUT.mkdir(parents=True)
    (OUT / 'empty').mkdir()
    results = []
    javac_specs = [('javac8', base.J8, ['-source', '8', '-target', '8']),
                   ('javac23', base.J23, ['--release', '8'])]
    for ordinal, old in enumerate(rows):
        row = dict(old)
        row['_ordinal'] = ordinal
        dest, cp, cp_entries = make_cp_for_target(row, war_cp)
        src = Path(row['source']['path'])
        source_pin = verify_pin(row['source'])
        (dest / 'source-pin.json').write_text(json.dumps(source_pin, indent=2) + '\n')
        compilers = {}
        for compiler, jdk, flags in javac_specs:
            output = dest / compiler
            output.mkdir()
            command = [jdk / 'bin/javac'] + flags + ['-proc:none', '-Xlint:-options',
                       '-classpath', cp, '-sourcepath', str(OUT / 'empty'), '-implicit:none',
                       '-d', str(output), str(src)]
            # Do not run anything besides javac. Existing baseline run() strips ambient
            # Java/Rust options, records exact command/cwd/env/exit/timing/output.
            meta, _ = base.run(command, dest / f'compile-{compiler}')
            compilers[compiler] = meta['exit_code']
        result = {'artifact': old['artifact'], 'internal_name': old['name'],
                  'source_pin': source_pin, 'classpath_entries': len(cp_entries),
                  'removed_target_family_entry_count': sum(len(e['removed_target_family_entries']) for e in cp_entries),
                  'compilers': compilers, 'semantic_validation': 'not_attempted',
                  'target_execution': 'not_attempted'}
        (dest / 'result.json').write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n')
        results.append(result)
    (OUT / 'source-target-family-token-audit.json').write_text(json.dumps(
        source_reference_audit(rows), ensure_ascii=False, indent=2) + '\n')
    (OUT / 'results.json').write_text(json.dumps({
        'schema': 'jarde-s0-real-class-isolated-compile-v1',
        'source_manifest': file_pin(INPUT), 'war_classpath_manifest': file_pin(WAR_CP),
        'rule': 'For each source, remove only that exact target internal class and its $* nested family from all classpath entries; retain classpath order and all other dependencies.',
        'compilers': {'javac8': {'path': str(base.J8 / 'bin/javac'), 'flags': ['-source', '8', '-target', '8']},
                      'javac23': {'path': str(base.J23 / 'bin/javac'), 'flags': ['--release', '8']}},
        'compilation_semantics': 'source unchanged; -proc:none -Xlint:-options -sourcepath empty -implicit:none; only compilation is attempted',
        'semantic_validation': 'not_attempted', 'target_execution': 'not_attempted',
        'results': results}, ensure_ascii=False, indent=2) + '\n')
    print(f'wrote {OUT}: {len(results)} source rows; javac invocations recorded in per-sample meta files')


if __name__ == '__main__':
    main()
