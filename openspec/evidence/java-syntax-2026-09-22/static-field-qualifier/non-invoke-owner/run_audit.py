"""Source-only owner-mismatch replay for an aload/pop static-call qualifier."""

from pathlib import Path
import hashlib
import json
import re
import shutil
import struct
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
CLI = Path('/tmp/jarde-cli-null-root-final')
EXPECTED_CLI = '30481c1449b5a1869773d21b934789d42bb1d6a9cab8d81f873d50055af4e34c'
PROBE = HERE / 'NonInvokeQualifierProbe.java'
RUNNER = HERE / 'NonInvokeQualifierRunner.java'
HELPERS = [HERE / 'Base.java', HERE / 'Child.java', HERE / 'Other.java']


def run(args, label):
    result = subprocess.run(args, capture_output=True, text=True, timeout=90)
    (HERE / f'{label}.stdout').write_text(result.stdout)
    (HERE / f'{label}.stderr').write_text(result.stderr)
    (HERE / f'{label}.status').write_text(f'{result.returncode}\n')
    return result


def cp_entries(data):
    count = struct.unpack_from('>H', data, 8)[0]
    entries = {}
    cursor = 10
    index = 1
    while index < count:
        tag = data[cursor]
        cursor += 1
        if tag == 1:
            size = struct.unpack_from('>H', data, cursor)[0]
            cursor += 2
            raw = data[cursor:cursor + size]
            entries[index] = {'tag': tag, 'value': raw.decode('utf-8'), 'start': cursor, 'length': size}
            cursor += size
        elif tag in (3, 4):
            entries[index] = {'tag': tag}
            cursor += 4
        elif tag in (5, 6):
            entries[index] = {'tag': tag}
            cursor += 8
            index += 1
        elif tag in (7, 8, 16, 19, 20):
            entries[index] = {'tag': tag, 'a': struct.unpack_from('>H', data, cursor)[0]}
            cursor += 2
        elif tag in (9, 10, 11, 12, 17, 18):
            a, b = struct.unpack_from('>HH', data, cursor)
            entries[index] = {'tag': tag, 'a': a, 'b': b}
            cursor += 4
        elif tag == 15:
            entries[index] = {'tag': tag}
            cursor += 3
        else:
            raise ValueError(f'unknown constant-pool tag {tag}')
        index += 1
    return entries


def utf(entries, index):
    entry = entries[index]
    assert entry['tag'] == 1
    return entry['value']


def find_methodref(data):
    entries = cp_entries(data)
    matches = []
    for index, entry in entries.items():
        if entry['tag'] not in (10, 11):
            continue
        owner_class = entries[entry['a']]
        owner = utf(entries, owner_class['a'])
        name_and_type = entries[entry['b']]
        name = utf(entries, name_and_type['a'])
        descriptor = utf(entries, name_and_type['b'])
        if (owner, name, descriptor) == ('Child', 'ping', '()I'):
            matches.append({'index': index, 'owner': owner, 'name': name, 'descriptor': descriptor,
                            'owner_utf_index': owner_class['a']})
    assert len(matches) == 1, matches
    return entries, matches[0]


def methodref_owner(data, cp_index):
    entries = cp_entries(data)
    entry = entries[cp_index]
    owner = utf(entries, entries[entry['a']]['a'])
    name_and_type = entries[entry['b']]
    return {'owner': owner, 'name': utf(entries, name_and_type['a']), 'descriptor': utf(entries, name_and_type['b'])}


def compile_generated(work, label, generated):
    out = work / label
    out.mkdir(exist_ok=True)
    package_match = re.search(r'^package\s+([^;]+);', generated.read_text(), re.MULTILINE)
    package = package_match.group(1) if package_match else None
    probe = out / 'NonInvokeQualifierProbe.java'
    shutil.copy2(generated, probe)
    runner = out / 'NonInvokeQualifierRunner.java'
    runner_source = RUNNER.read_text()
    if package and not re.search(r'^package\s', runner_source, re.MULTILINE):
        runner_source = f'package {package};\n\n' + runner_source
    runner.write_text(runner_source)
    sources = [str(probe), str(runner)]
    for helper in HELPERS:
        copied = out / helper.name
        text = helper.read_text()
        if package:
            text = f'package {package};\n\n' + text
        copied.write_text(text)
        sources.append(str(copied))
    classes = out / 'classes'
    compiled = run(['javac', '--release', '8', '-g:none', '-d', str(classes), *sources], f'{label}-javac')
    result = {'javac': compiled.returncode}
    if compiled.returncode == 0:
        main = f'{package}.NonInvokeQualifierRunner' if package else 'NonInvokeQualifierRunner'
        executed = run(['java', '-Xverify:all', '-cp', str(classes), main], f'{label}-runtime')
        result.update(runtime=executed.returncode, output=executed.stdout.splitlines())
    return result


def decompile_jadx(work, class_file, label):
    out = work / f'{label}-jadx'
    result = run(['jadx', '--no-res', '-d', str(out), str(class_file)], f'{label}-jadx-decompile')
    assert result.returncode == 0
    generated = next(out.rglob('NonInvokeQualifierProbe.java'))
    saved = HERE / f'{label}-jadx.java.txt'
    shutil.copy2(generated, saved)
    return saved


def decompile_jarde(class_file, label):
    result = run([
        str(CLI), 'class-source', '--input', str(class_file), '--class', 'NonInvokeQualifierProbe',
        '--policy', 'single-class', '--release', '8', '--format', 'text',
    ], f'{label}-jarde-cli')
    assert result.returncode == 0
    saved = HERE / f'{label}-jarde.java.txt'
    saved.write_text(result.stdout)
    return saved


def main():
    cli_before = hashlib.sha256(CLI.read_bytes()).hexdigest()
    assert cli_before == EXPECTED_CLI
    with tempfile.TemporaryDirectory(prefix='jarde-non-invoke-owner-') as raw:
        work = Path(raw)
        compiled = work / 'source-compiled'
        compiled.mkdir()
        source_compile = run([
            'javac', '--release', '8', '-g:none', '-d', str(compiled),
            *map(str, [PROBE, RUNNER, *HELPERS]),
        ], 'source-javac')
        assert source_compile.returncode == 0
        original_class = compiled / 'NonInvokeQualifierProbe.class'
        saved_original = HERE / 'NonInvokeQualifierProbe.class'
        shutil.copy2(original_class, saved_original)
        original_data = original_class.read_bytes()
        entries, methodref = find_methodref(original_data)
        assert methodref_owner(original_data, methodref['index']) == {'owner': 'Child', 'name': 'ping', 'descriptor': '()I'}
        javap_original = run(['javap', '-c', '-p', str(original_class)], 'original-javap')
        source_runtime = run(['java', '-Xverify:all', '-cp', str(compiled), 'NonInvokeQualifierRunner'], 'source-runtime')
        assert source_runtime.returncode == 0

        # Replace only the CONSTANT_Class name behind the call Methodref. The method name and
        # descriptor remain the same; `arg` is popped before invokestatic, so verification has no
        # receiver-to-owner type relation to enforce.
        owner_utf = entries[methodref['owner_utf_index']]
        old_owner = owner_utf['value'].encode('utf-8')
        new_owner = b'Other'
        assert len(old_owner) == len(new_owner)
        patched_data = bytearray(original_data)
        patched_data[owner_utf['start']:owner_utf['start'] + owner_utf['length']] = new_owner
        patched_data = bytes(patched_data)
        changed_offsets = [i for i, (old, new) in enumerate(zip(original_data, patched_data)) if old != new]
        assert len(changed_offsets) == 5
        patched_class = HERE / 'NonInvokeQualifierProbe.owner-other.class'
        patched_class.write_bytes(patched_data)
        patched_hash = hashlib.sha256(patched_data).hexdigest()
        assert methodref_owner(patched_data, methodref['index']) == {'owner': 'Other', 'name': 'ping', 'descriptor': '()I'}

        patched_run_dir = work / 'patched-jvm'
        patched_run_dir.mkdir()
        for helper in ['Base.class', 'Child.class', 'Other.class', 'NonInvokeQualifierRunner.class']:
            shutil.copy2(compiled / helper, patched_run_dir / helper)
        shutil.copy2(patched_class, patched_run_dir / 'NonInvokeQualifierProbe.class')
        patched_runtime = run(['java', '-Xverify:all', '-cp', str(patched_run_dir), 'NonInvokeQualifierRunner'], 'patched-runtime')
        assert patched_runtime.returncode == 0
        javap_patched = run(['javap', '-c', '-p', str(patched_class)], 'patched-javap')

        artifacts = {}
        for case, class_file in [('original', original_class), ('patched', patched_class)]:
            jadx_source = decompile_jadx(work, class_file, case)
            jarde_source = decompile_jarde(class_file, case)
            artifacts[case] = {
                'jadx_source': jadx_source,
                'jarde_source': jarde_source,
                'jadx': compile_generated(work, f'{case}-jadx', jadx_source),
                'jarde': compile_generated(work, f'{case}-jarde', jarde_source),
            }

        cli_after = hashlib.sha256(CLI.read_bytes()).hexdigest()
        assert cli_after == EXPECTED_CLI
        summary = {
            'cli_sha256': cli_before,
            'source_sha256': {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in [PROBE, RUNNER, *HELPERS]},
            'javac': subprocess.run(['javac', '-version'], capture_output=True, text=True).stdout.strip(),
            'class_bytes': len(original_data),
            'original_class_sha256': hashlib.sha256(original_data).hexdigest(),
            'methodref_cp_index': methodref['index'],
            'original_methodref': methodref_owner(original_data, methodref['index']),
            'patched_methodref': methodref_owner(patched_data, methodref['index']),
            'owner_patch_changed_byte_offsets': changed_offsets,
            'patched_class_sha256': patched_hash,
            'source_compile': source_compile.returncode,
            'source_runtime': {'status': source_runtime.returncode, 'output': source_runtime.stdout.splitlines()},
            'patched_verify_runtime': {'status': patched_runtime.returncode, 'output': patched_runtime.stdout.splitlines()},
            'original_code': javap_original.stdout,
            'patched_code': javap_patched.stdout,
            'original_code_attribute_count': javap_original.stdout.count('    Code:'),
            'patched_code_attribute_count': javap_patched.stdout.count('    Code:'),
            'decompilers': {
                case: {
                    'jadx': data['jadx'],
                    'jarde': data['jarde'],
                    'jadx_call_line': next((line.strip() for line in data['jadx_source'].read_text().splitlines() if '.ping(' in line), None),
                    'jarde_call_line': next((line.strip() for line in data['jarde_source'].read_text().splitlines() if '.ping(' in line), None),
                }
                for case, data in artifacts.items()
            },
            'cli_sha256_after': cli_after,
        }
        (HERE / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
        print(json.dumps(summary, indent=2))


if __name__ == '__main__':
    main()
