"""Source-only static-qualifier boundary replay, including verifier-valid Methodref owner patches."""

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
PROBE = HERE / 'QualifierBoundaryProbe.java'
RUNNER = HERE / 'QualifierBoundaryRunner.java'


def run(args, label):
    result = subprocess.run(args, capture_output=True, text=True, timeout=90)
    (HERE / f'{label}.stdout').write_text(result.stdout)
    (HERE / f'{label}.stderr').write_text(result.stderr)
    (HERE / f'{label}.status').write_text(f'{result.returncode}\n')
    return result


def package_of(source):
    match = re.search(r'^package\s+([^;]+);', source.read_text(), re.MULTILINE)
    return match.group(1) if match else None


def compile_pair(work, label, probe_source, runner_source=RUNNER, extra_sources=(), target_name='QualifierBoundaryProbe.java'):
    out = work / f'{label}-compile'
    out.mkdir(exist_ok=True)
    probe = out / target_name
    shutil.copy2(probe_source, probe)
    package = package_of(probe_source)
    runner = out / runner_source.name
    runner_text = runner_source.read_text()
    if package and not re.search(r'^package\s', runner_text, re.MULTILINE):
        runner_text = f'package {package};\n\n' + runner_text
    runner.write_text(runner_text)
    sources = [str(probe), str(runner)]
    for source in extra_sources:
        copied = out / source.name
        shutil.copy2(source, copied)
        if package and not re.search(r'^package\s', copied.read_text(), re.MULTILINE):
            copied.write_text(f'package {package};\n\n' + copied.read_text())
        sources.append(str(copied))
    classes = out / 'classes'
    javac = run(['javac', '--release', '8', '-g:none', '-d', str(classes), *sources], f'{label}-javac')
    result = {'javac': javac.returncode}
    if javac.returncode == 0:
        main = f'{package}.{runner.stem}' if package else runner.stem
        executed = run(['java', '-Xverify:all', '-cp', str(classes), main], f'{label}-runtime')
        result.update(runtime=executed.returncode, output=executed.stdout.splitlines())
    return result


def cp_entries(data):
    count = struct.unpack_from('>H', data, 8)[0]
    entries = {}
    cursor = 10
    index = 1
    while index < count:
        tag = data[cursor]
        start = cursor
        cursor += 1
        if tag == 1:
            size = struct.unpack_from('>H', data, cursor)[0]
            cursor += 2
            raw = data[cursor:cursor + size]
            entries[index] = {'tag': tag, 'value': raw.decode('utf-8'), 'start': cursor, 'length': size}
            cursor += size
        elif tag in (3, 4):
            cursor += 4
            entries[index] = {'tag': tag}
        elif tag in (5, 6):
            cursor += 8
            entries[index] = {'tag': tag}
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
            raise ValueError(f'unknown constant-pool tag {tag} at #{index}')
        index += 1
    return entries


def utf(entries, index):
    entry = entries[index]
    assert entry['tag'] == 1
    return entry['value']


def methodref(data, name):
    entries = cp_entries(data)
    candidates = []
    for index, entry in entries.items():
        if entry['tag'] not in (10, 11):
            continue
        owner = utf(entries, entries[entry['a']]['a'])
        nat = entries[entry['b']]
        method_name = utf(entries, nat['a'])
        descriptor = utf(entries, nat['b'])
        if method_name == name and descriptor == '()I':
            candidates.append({'index': index, 'owner': owner, 'name': method_name, 'descriptor': descriptor,
                               'owner_utf_index': entries[entry['a']]['a'], 'name_utf_index': nat['a']})
    assert len(candidates) == 1, candidates
    return entries, candidates[0]


def replace_utf(data, entries, index, replacement):
    entry = entries[index]
    old = entry['value'].encode('utf-8')
    new = replacement.encode('utf-8')
    assert len(old) == len(new), (entry['value'], replacement)
    patched = bytearray(data)
    patched[entry['start']:entry['start'] + entry['length']] = new
    return bytes(patched)


def normalize_code(javap, method):
    match = re.search(rf'^  public static [^\n]*\b{re.escape(method)}\([^\n]*$', javap, re.MULTILINE)
    assert match, f'missing method {method}'
    tail = javap[match.end():]
    lines = []
    for line in tail.splitlines():
        if not line.strip():
            if lines:
                break
            continue
        instruction = re.match(r'^\s+\d+:\s+(.*)$', line)
        if instruction:
            body = instruction.group(1)
            if '//' in body:
                op, comment = body.split('//', 1)
                body = re.sub(r'#\d+', '#', op.strip()) + f' // {comment.strip()}'
            else:
                body = re.sub(r'#\d+', '#', body.strip())
            lines.append(body)
    return lines


def owner_fixture_sources():
    files = {
        'Base.java': 'class Base { static int ping() { return 31; } }\n',
        'Child.java': 'class Child extends Base {}\n',
        'Peer.java': 'class Peer { static int ping() { return 41; } }\n',
        'Evil.java': 'class Evil { static int pong() { return 51; } }\n',
        'Other.java': 'class Other { static int ping() { return 41; } }\n',
        'EvilX.java': 'class EvilX { static int pong() { return 51; } }\n',
        'OwnerBoundaryProbe.java': '''public final class OwnerBoundaryProbe {
    static Child receiver() { return null; }
    public static int call() { return receiver().ping(); }
}
''',
        'OwnerBoundaryRunner.java': '''public final class OwnerBoundaryRunner {
    public static void main(String[] args) { System.out.println(OwnerBoundaryProbe.call()); }
}
''',
    }
    paths = []
    for name, contents in files.items():
        path = HERE / name
        path.write_text(contents)
        paths.append(path)
    return paths


def compile_owner(work, label, probe, helper_sources):
    out = work / f'{label}-compile'
    out.mkdir(exist_ok=True)
    package = package_of(probe)
    copied = []
    for source in [probe, *helper_sources]:
        dest = out / ('OwnerBoundaryProbe.java' if source == probe else source.name)
        shutil.copy2(source, dest)
        if package and not re.search(r'^package\s', dest.read_text(), re.MULTILINE):
            dest.write_text(f'package {package};\n\n' + dest.read_text())
        copied.append(str(dest))
    classes = out / 'classes'
    javac = run(['javac', '--release', '8', '-g:none', '-d', str(classes), *copied], f'{label}-javac')
    result = {'javac': javac.returncode}
    if javac.returncode == 0:
        main = f'{package}.OwnerBoundaryRunner' if package else 'OwnerBoundaryRunner'
        executed = run(['java', '-Xverify:all', '-cp', str(classes), main], f'{label}-runtime')
        result.update(runtime=executed.returncode, output=executed.stdout.splitlines())
    return result


def main():
    assert hashlib.sha256(CLI.read_bytes()).hexdigest() == EXPECTED_CLI
    results = {'cli_sha256': EXPECTED_CLI}
    with tempfile.TemporaryDirectory(prefix='jarde-static-qualifier-boundary-') as raw:
        work = Path(raw)
        original_dir = work / 'original'
        original_dir.mkdir()
        original_compile = run(['javac', '--release', '8', '-g:none', '-d', str(original_dir), str(PROBE), str(RUNNER)], 'original-javac')
        assert original_compile.returncode == 0
        original_class = original_dir / 'QualifierBoundaryProbe.class'
        shutil.copy2(original_class, HERE / 'QualifierBoundaryProbe.class')
        original_javap = run(['javap', '-c', '-p', str(original_class)], 'original-javap')
        assert original_javap.returncode == 0
        original_run = run(['java', '-Xverify:all', '-cp', str(original_dir), 'QualifierBoundaryRunner'], 'original-runtime')
        assert original_run.returncode == 0
        (HERE / 'original.class.sha256').write_text(hashlib.sha256(original_class.read_bytes()).hexdigest() + '\n')

        generated = run([str(CLI), 'class-source', '--input', str(original_class), '--class', 'QualifierBoundaryProbe', '--policy', 'single-class', '--release', '8', '--format', 'text'], 'jarde-cli')
        assert generated.returncode == 0
        jarde_source = HERE / 'jarde.java.txt'
        jarde_source.write_text(generated.stdout)
        jarde = compile_pair(work, 'jarde', jarde_source)
        jadx_root = work / 'jadx'
        jadx = run(['jadx', '--no-res', '-d', str(jadx_root), str(original_class)], 'jadx')
        assert jadx.returncode == 0
        jadx_source = next(jadx_root.rglob('QualifierBoundaryProbe.java'))
        (HERE / 'jadx.java.txt').write_text(jadx_source.read_text())
        jadx_result = compile_pair(work, 'jadx', jadx_source)
        originals = original_run.stdout.splitlines()

        methods = ['staticRead', 'qualifiedCallReturn', 'plainCallReturn', 'qualifiedCallStatement',
                   'plainCallStatements', 'qualifiedStaticWrite', 'fieldWriteWithQualifiedRhs',
                   'plainSequenceStaticWrite', 'qualifiedStaticWriteThrowingRhs', 'qualifiedCallThrowingReceiver']
        normalized = {name: normalize_code(original_javap.stdout, name) for name in methods}
        code_pairs = {
            'qualified_call_vs_plain_return': ['qualifiedCallReturn', 'plainCallReturn'],
            'qualified_call_vs_plain_statement': ['qualifiedCallStatement', 'plainCallStatements'],
            'static_field_qualifier_vs_qualified_rhs': ['qualifiedStaticWrite', 'fieldWriteWithQualifiedRhs'],
            'static_field_qualifier_vs_plain_sequence': ['qualifiedStaticWrite', 'plainSequenceStaticWrite'],
        }
        results.update({
            'probe_source_sha256': hashlib.sha256(PROBE.read_bytes()).hexdigest(),
            'runner_source_sha256': hashlib.sha256(RUNNER.read_bytes()).hexdigest(),
            'class_bytes': original_class.stat().st_size,
            'class_sha256': hashlib.sha256(original_class.read_bytes()).hexdigest(),
            'code_methods': original_javap.stdout.count('    Code:'),
            'original_runtime': original_run.stdout.splitlines(),
            'jarde': jarde,
            'jadx': jadx_result,
            'code': normalized,
            'code_pairs_identical': {label: normalized[pair[0]] == normalized[pair[1]] for label, pair in code_pairs.items()},
            'runtime_equal': {'jarde': jarde.get('output') == originals if jarde.get('runtime') == 0 else None,
                              'jadx': jadx_result.get('output') == originals if jadx_result.get('runtime') == 0 else None},
        })

        # A valid Java source call through Child resolves to Base.ping. Repointing only the
        # CONSTANT_Methodref owner to Peer remains verifier-valid, while expression-qualified
        # reconstructed Java still resolves Child.ping to Base.ping.
        owner_sources = owner_fixture_sources()
        owner_probe = HERE / 'OwnerBoundaryProbe.java'
        owner_helpers = [path for path in owner_sources if path.name not in ('OwnerBoundaryProbe.java', 'OwnerBoundaryRunner.java')]
        owner_original_dir = work / 'owner-original'
        owner_original_dir.mkdir()
        owner_source_compile = run(['javac', '--release', '8', '-g:none', '-d', str(owner_original_dir), *map(str, owner_sources)], 'owner-original-javac')
        assert owner_source_compile.returncode == 0
        owner_class = owner_original_dir / 'OwnerBoundaryProbe.class'
        shutil.copy2(owner_class, HERE / 'OwnerBoundaryProbe.class')
        owner_data = owner_class.read_bytes()
        entries, ref = methodref(owner_data, 'ping')
        results['owner_source_methodref'] = ref
        results['owner_source'] = {
            'source_sha256': {path.name: hashlib.sha256(path.read_bytes()).hexdigest() for path in owner_sources},
            'class_bytes': owner_class.stat().st_size,
            'class_sha256': hashlib.sha256(owner_data).hexdigest(),
        }
        owner_javap = run(['javap', '-c', '-p', str(owner_class)], 'owner-original-javap')
        results['owner_source_code'] = normalize_code(owner_javap.stdout, 'call')
        owner_original_run = run(['java', '-Xverify:all', '-cp', str(owner_original_dir), 'OwnerBoundaryRunner'], 'owner-original-runtime')
        results['owner_source_runtime'] = {'status': owner_original_run.returncode, 'output': owner_original_run.stdout.splitlines()}

        owner_generated = run([str(CLI), 'class-source', '--input', str(owner_class), '--class', 'OwnerBoundaryProbe', '--policy', 'single-class', '--release', '8', '--format', 'text'], 'owner-jarde-cli')
        assert owner_generated.returncode == 0
        (HERE / 'owner-jarde.java.txt').write_text(owner_generated.stdout)

        patches = {}
        replacements = {'Base': [('Peer', 'ping'), ('Evil', 'pong')], 'Child': [('Other', 'ping'), ('EvilX', 'pong')]}
        assert ref['owner'] in replacements, f'unexpected javac Methodref owner {ref["owner"]}'
        for label, (owner_name, method_name) in zip(('owner-only-peer', 'unrelated-owner-and-name'), replacements[ref['owner']]):
            patched = owner_data
            current_entries = cp_entries(patched)
            # The compiler's symbolic owner may be Base or Child; keep the patch tied to the
            # Methodref actually used by `call`, as confirmed from its javap BCI.
            patched = replace_utf(patched, current_entries, ref['owner_utf_index'], owner_name)
            current_entries = cp_entries(patched)
            patched = replace_utf(patched, current_entries, ref['name_utf_index'], method_name)
            patched_path = HERE / f'{label}.class'
            patched_path.write_bytes(patched)
            changed_offsets = [offset for offset, pair in enumerate(zip(owner_data, patched)) if pair[0] != pair[1]]
            target_class = work / label
            target_class.mkdir()
            shutil.copy2(patched_path, target_class / 'OwnerBoundaryProbe.class')
            for helper in owner_helpers:
                shutil.copy2(owner_original_dir / f'{helper.stem}.class', target_class / f'{helper.stem}.class')
            shutil.copy2(owner_original_dir / 'OwnerBoundaryRunner.class', target_class / 'OwnerBoundaryRunner.class')
            verified = run(['java', '-Xverify:all', '-cp', str(target_class), 'OwnerBoundaryRunner'], f'{label}-patched-runtime')
            assert verified.returncode == 0
            javap = run(['javap', '-c', '-p', str(target_class / 'OwnerBoundaryProbe.class')], f'{label}-patched-javap')
            decompiled = run([str(CLI), 'class-source', '--input', str(target_class / 'OwnerBoundaryProbe.class'), '--class', 'OwnerBoundaryProbe', '--policy', 'single-class', '--release', '8', '--format', 'text'], f'{label}-jarde-cli')
            assert decompiled.returncode == 0
            jarde_path = HERE / f'{label}-jarde.java.txt'
            jarde_path.write_text(decompiled.stdout)
            compiled = compile_owner(work, label, jarde_path, [*owner_helpers, next(p for p in owner_sources if p.name == 'OwnerBoundaryRunner.java')])
            jadx_out = work / f'{label}-jadx'
            jadx_run = run(['jadx', '--no-res', '-d', str(jadx_out), str(patched_path)], f'{label}-jadx-decompile')
            assert jadx_run.returncode == 0
            jadx_probe = next(jadx_out.rglob('OwnerBoundaryProbe.java'))
            jadx_saved = HERE / f'{label}-jadx.java.txt'
            shutil.copy2(jadx_probe, jadx_saved)
            jadx_compiled = compile_owner(work, f'{label}-jadx', jadx_saved, [*owner_helpers, next(p for p in owner_sources if p.name == 'OwnerBoundaryRunner.java')])
            patches[label] = {
                'class_sha256': hashlib.sha256(patched).hexdigest(),
                'changed_byte_offsets': changed_offsets,
                'methodref_after_patch': methodref(patched, method_name)[1],
                'verified_runtime': {'status': verified.returncode, 'output': verified.stdout.splitlines()},
                'code': normalize_code(javap.stdout, 'call'),
                'jarde': compiled,
                'jadx': jadx_compiled,
                'jarde_call_lines': [line.strip() for line in decompiled.stdout.splitlines() if 'ping(' in line or 'pong(' in line],
                'jadx_call_lines': [line.strip() for line in jadx_probe.read_text().splitlines() if 'ping(' in line or 'pong(' in line],
            }
        results['owner_patches'] = patches

        # A Java 8 interface static method has no expression-qualified source form. This legal
        # producer;pop;invokestatic InterfaceMethodref shape checks the exact generic static-call
        # qualifier path without adding an illegal method to the positive whole-class fixture.
        interface_original_dir = work / 'interface-original'
        interface_original_dir.mkdir()
        interface_sources = [HERE / 'InterfaceStaticOwner.java', HERE / 'InterfaceBoundaryProbe.java', HERE / 'InterfaceBoundaryRunner.java']
        interface_compile = run(['javac', '--release', '8', '-g:none', '-d', str(interface_original_dir), *map(str, interface_sources)], 'interface-original-javac')
        assert interface_compile.returncode == 0
        interface_class = interface_original_dir / 'InterfaceBoundaryProbe.class'
        shutil.copy2(interface_class, HERE / 'InterfaceBoundaryProbe.class')
        interface_javap = run(['javap', '-c', '-p', str(interface_class)], 'interface-original-javap')
        interface_runtime = run(['java', '-Xverify:all', '-cp', str(interface_original_dir), 'InterfaceBoundaryRunner'], 'interface-original-runtime')
        assert interface_javap.returncode == 0 and interface_runtime.returncode == 0
        interface_jarde = run([str(CLI), 'class-source', '--input', str(interface_class), '--class', 'InterfaceBoundaryProbe', '--policy', 'single-class', '--release', '8', '--format', 'text'], 'interface-jarde-cli')
        assert interface_jarde.returncode == 0
        interface_jarde_source = HERE / 'interface-jarde.java.txt'
        interface_jarde_source.write_text(interface_jarde.stdout)
        interface_jarde_result = compile_pair(work, 'interface-jarde', interface_jarde_source,
                                               HERE / 'InterfaceBoundaryRunner.java', [HERE / 'InterfaceStaticOwner.java'],
                                               target_name='InterfaceBoundaryProbe.java')
        interface_jadx_out = work / 'interface-jadx'
        interface_jadx = run(['jadx', '--no-res', '-d', str(interface_jadx_out), str(interface_class)], 'interface-jadx-decompile')
        assert interface_jadx.returncode == 0
        interface_jadx_probe = next(interface_jadx_out.rglob('InterfaceBoundaryProbe.java'))
        interface_jadx_saved = HERE / 'interface-jadx.java.txt'
        shutil.copy2(interface_jadx_probe, interface_jadx_saved)
        interface_jadx_result = compile_pair(work, 'interface-jadx', interface_jadx_saved,
                                              HERE / 'InterfaceBoundaryRunner.java', [HERE / 'InterfaceStaticOwner.java'],
                                              target_name='InterfaceBoundaryProbe.java')
        illegal_dir = work / 'interface-illegal-source'
        illegal_dir.mkdir()
        illegal_compile = run(['javac', '--release', '8', '-g:none', '-d', str(illegal_dir),
                               str(HERE / 'IllegalInterfaceQualifier.java'), str(HERE / 'InterfaceStaticOwner.java')],
                              'interface-expression-qualifier-illegal-javac')
        results['interface_static_boundary'] = {
            'source_sha256': {path.name: hashlib.sha256(path.read_bytes()).hexdigest() for path in interface_sources},
            'class_bytes': interface_class.stat().st_size,
            'class_sha256': hashlib.sha256(interface_class.read_bytes()).hexdigest(),
            'code_methods': interface_javap.stdout.count('    Code:'),
            'code': normalize_code(interface_javap.stdout, 'call'),
            'original_runtime': {'status': interface_runtime.returncode, 'output': interface_runtime.stdout.splitlines()},
            'jarde': interface_jarde_result,
            'jadx': interface_jadx_result,
            'illegal_expression_qualifier_javac': illegal_compile.returncode,
            'illegal_expression_qualifier_source_sha256': hashlib.sha256((HERE / 'IllegalInterfaceQualifier.java').read_bytes()).hexdigest(),
            'illegal_expression_qualifier_stderr': illegal_compile.stderr.splitlines(),
            'jarde_source_lines': [line.strip() for line in interface_jarde.stdout.splitlines() if 'receiver()' in line or 'InterfaceStaticOwner.value' in line],
            'jadx_source_lines': [line.strip() for line in interface_jadx_probe.read_text().splitlines() if 'receiver()' in line or 'InterfaceStaticOwner.value' in line],
        }

    results['cli_sha256_after'] = hashlib.sha256(CLI.read_bytes()).hexdigest()
    assert results['cli_sha256_after'] == EXPECTED_CLI
    (HERE / 'summary.json').write_text(json.dumps(results, indent=2) + '\n')
    print(json.dumps(results, indent=2))


if __name__ == '__main__':
    main()
