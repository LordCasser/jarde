#!/usr/bin/env python3
"""Verifier-valid EM-01 physical neighbors; every root projection must refuse."""
import argparse
import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import zipfile

HERE = Path(__file__).resolve().parent
ROOT = (HERE / 'Generic.class').read_bytes()
CHILD = (HERE / 'Generic$A.class').read_bytes()


def replace_one(data, old, new):
    assert len(old) == len(new) and data.count(old) == 1
    return data.replace(old, new, 1)


def add_pool(data, entries, count_added):
    count = int.from_bytes(data[8:10], 'big')
    offset, index = 10, 1
    while index < count:
        tag = data[offset]
        offset += 1
        if tag == 1:
            length = int.from_bytes(data[offset:offset + 2], 'big')
            offset += 2 + length
        elif tag in (3, 4, 9, 10, 11, 12, 17, 18):
            offset += 4
        elif tag in (5, 6):
            offset += 8
            index += 1
        elif tag in (7, 8, 16, 19, 20):
            offset += 2
        elif tag == 15:
            offset += 3
        else:
            raise AssertionError(f'unexpected constant-pool tag {tag}')
        index += 1
    return (data[:8] + (count + count_added).to_bytes(2, 'big') + data[10:offset]
            + entries + data[offset:]), count


def command(args, *, check=True):
    result = subprocess.run([str(arg) for arg in args], capture_output=True, text=True, timeout=60)
    if check and result.returncode:
        raise RuntimeError(f'{args}: {result.stdout}{result.stderr}')
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--jarde', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    out = args.out.resolve()
    if out.exists() and any(out.iterdir()):
        parser.error('--out must be empty')
    out.mkdir(parents=True, exist_ok=True)
    old_row = bytes([0, 1, 0, 7, 0, 25, 0, 27, 4, 9])
    body = bytes([0x2a, 0x2b, 0xc0, 0, 7, 0xb6, 0, 9, 0xac])
    target_pool, target_index = add_pool(CHILD, bytes([10, 0, 2, 0, 10]), 1)
    cast_name = b'em01/GenXric'
    cast_pool, cast_name_index = add_pool(CHILD,
        bytes([1, 0, len(cast_name)]) + cast_name + bytes([7, 0, 28]), 2)
    cases = {
        'wrong-self-row': replace_one(CHILD, old_row, bytes([0, 1, 0, 7, 0, 2, 0, 27, 4, 9])),
        'wrong-interface-signature': replace_one(CHILD, b'java/lang/Comparable<', b'java/lang/ComparablE<'),
        'unknown-field-T': replace_one(CHILD, bytes([0, 3, 84, 84, 59]), bytes([0, 3, 84, 85, 59])),
        'unknown-method-T': replace_one(CHILD, b'(Lem01/Generic$A<TT;>;)I', b'(Lem01/Generic$A<TU;>;)I'),
        'bridge-extra-effect': replace_one(CHILD, body, bytes([0x2a, 0x59, 0xc0, 0, 7, 0xb6, 0, 9, 0xac])),
        'bridge-wrong-target': replace_one(target_pool, bytes([0xb6, 0, 9, 0xac]), bytes([0xb6, 0, target_index, 0xac])),
        'bridge-wrong-cast': replace_one(cast_pool, bytes([0xc0, 0, 7, 0xb6]), bytes([0xc0, 0, cast_name_index + 1, 0xb6])),
    }
    with tempfile.TemporaryDirectory(prefix='jarde-em01-negative-') as temp:
        work = Path(temp)
        classes = work / 'classes'
        (classes / 'em01').mkdir(parents=True)
        (classes / 'em01' / 'Generic.class').write_bytes(ROOT)
        (classes / 'em01' / 'Generic$A.class').write_bytes(CHILD)
        (work / 'VerifyLoad.java').write_text(
            'public class VerifyLoad { public static void main(String[] a) throws Exception '
            '{ System.out.println(Class.forName("em01.Generic$A").getName()); } }')
        (work / 'BridgeProbe.java').write_text(
            'import em01.Generic; public class BridgeProbe { @SuppressWarnings({"rawtypes","unchecked"}) '
            'public static void main(String[] a) { Generic.A<String> v = new Generic.A<String>() {}; '
            'System.out.println(((Comparable)v).compareTo(v)); } }')
        (work / 'GenXric.java').write_text(
            'package em01; public class GenXric extends Generic.A<String> {}')
        command(['javac', '--release', '8', '-g:none', '-Xlint:-options', '-cp', classes,
                 '-d', classes, work / 'VerifyLoad.java', work / 'BridgeProbe.java', work / 'GenXric.java',
                 HERE / 'BridgeRunner.java'])
        results = {}
        for name, child in cases.items():
            location = work / name
            shutil.copytree(classes, location)
            (location / 'em01' / 'Generic$A.class').write_bytes(child)
            loaded = command(['java', '-Xverify:all', '-cp', location, 'VerifyLoad'])
            if loaded.stdout.strip() != 'em01.Generic$A':
                raise RuntimeError(f'{name}: verifier load changed')
            archive = work / (name + '.jar')
            with zipfile.ZipFile(archive, 'w') as jar:
                jar.writestr('em01/Generic.class', ROOT)
                jar.writestr('em01/Generic$A.class', child)
            result = command([args.jarde, 'class-source', '--input', archive,
                              '--class', 'em01.Generic', '--policy', 'plain-jar',
                              '--release', '8', '--evidence', 'essential', '--format', 'json'])
            report = json.loads(result.stdout)
            state = report['member_family']['projection']['state'] if report['member_family']['state'] == 'prepared' else report['member_family']['state']
            if state == 'projected' or 'class A<T>' in report['text']:
                raise RuntimeError(f'{name}: root projected an unproved generic child')
            results[name] = {'verify_load': loaded.stdout.strip(), 'jarde_family': state}
            if name == 'bridge-extra-effect':
                observed = command(['java', '-Xverify:all', '-cp', location, 'em01.BridgeRunner'])
                if 'bridge-wrong=accepted' not in observed.stdout:
                    raise RuntimeError('bridge effect mutation did not change observable cast behavior')
                results[name]['behavior'] = 'bridge-wrong=accepted'
            elif name == 'bridge-wrong-target':
                observed = command(['java', '-Xverify:all', '-cp', location, 'em01.BridgeRunner'], check=False)
                if 'NoSuchMethodError' not in observed.stderr:
                    raise RuntimeError('wrong bridge target did not fail at invocation')
                results[name]['behavior'] = 'NoSuchMethodError'
            elif name == 'bridge-wrong-cast':
                observed = command(['java', '-Xverify:all', '-cp', location, 'BridgeProbe'], check=False)
                if 'ClassCastException' not in observed.stderr:
                    raise RuntimeError('wrong bridge cast did not reject a legal A value')
                results[name]['behavior'] = 'ClassCastException for legal A'
        root_use = work / 'root-use'
        root_use.mkdir()
        source = (HERE.parent.parent / 'java-syntax-2026-09-27' / 'em01-declarations' / 'input' / 'em01' / 'Generic.java').read_text()
        source = source.replace('\n}', '\n    public A<?> extra(A<?> value) { return value; }\n}')
        (root_use / 'Generic.java').write_text(source)
        command(['javac', '--release', '8', '-g:none', '-Xlint:-options', '-d', root_use, root_use / 'Generic.java'])
        archive = work / 'root-use.jar'
        with zipfile.ZipFile(archive, 'w') as jar:
            jar.write(root_use / 'em01' / 'Generic.class', 'em01/Generic.class')
            jar.write(root_use / 'em01' / 'Generic$A.class', 'em01/Generic$A.class')
        report = json.loads(command([args.jarde, 'class-source', '--input', archive,
            '--class', 'em01.Generic', '--policy', 'plain-jar', '--release', '8',
            '--evidence', 'essential', '--format', 'json']).stdout)
        if report['member_family']['projection']['state'] == 'projected':
            raise RuntimeError('root extra use was projected')
        results['root-extra-use'] = {'jarde_family': report['member_family']['projection']['state']}
        unrelated = work / 'root-unrelated'
        unrelated.mkdir()
        source = (HERE.parent.parent / 'java-syntax-2026-09-27' / 'em01-declarations' / 'input' / 'em01' / 'Generic.java').read_text()
        source = source.replace('\n}', '\n    public int ping() { return 1; }\n}')
        (unrelated / 'Generic.java').write_text(source)
        command(['javac', '--release', '8', '-g:none', '-Xlint:-options', '-d', unrelated,
                 unrelated / 'Generic.java'])
        archive = work / 'root-unrelated.jar'
        with zipfile.ZipFile(archive, 'w') as jar:
            jar.write(unrelated / 'em01' / 'Generic.class', 'em01/Generic.class')
            jar.write(unrelated / 'em01' / 'Generic$A.class', 'em01/Generic$A.class')
        report = json.loads(command([args.jarde, 'class-source', '--input', archive,
            '--class', 'em01.Generic', '--policy', 'plain-jar', '--release', '8',
            '--evidence', 'essential', '--format', 'json']).stdout)
        if report['member_family']['projection']['state'] != 'projected':
            raise RuntimeError('unrelated root method prevented declaration-only proof')
        results['root-unrelated-method'] = {'jarde_family': 'projected'}
    (out / 'summary.json').write_text(json.dumps(results, indent=2) + '\n')
    print(json.dumps(results, indent=2))


if __name__ == '__main__':
    main()
