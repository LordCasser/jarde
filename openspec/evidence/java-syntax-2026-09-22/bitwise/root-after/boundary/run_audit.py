#!/usr/bin/env python3
"""Reproduce source, legal descriptor, and stack-consumer boundary comparisons."""
from __future__ import annotations

import hashlib
import json
import os
import shutil
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[6]
EVIDENCE = Path(__file__).resolve().parent
WORK = Path('/tmp/jarde-bitwise-boundary-root-after')
CLI = Path('/tmp/jarde-cli-bitwise-root-after')
PROBE_SOURCE = EVIDENCE / 'BoundaryProbe.java'
RUNNER_SOURCE = EVIDENCE / 'BoundaryRunner.java'


def run(args: list[str], log: Path, cwd: Path | None = None) -> int:
    result = subprocess.run(args, cwd=cwd, capture_output=True, text=True, timeout=60)
    log.parent.mkdir(parents=True, exist_ok=True)
    log.write_text(result.stdout + result.stderr)
    return result.returncode


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def read_u2(data: bytes | bytearray, offset: int) -> int:
    return int.from_bytes(data[offset:offset + 2], 'big')


def read_u4(data: bytes | bytearray, offset: int) -> int:
    return int.from_bytes(data[offset:offset + 4], 'big')


def write_u4(data: bytearray, offset: int, value: int) -> None:
    data[offset:offset + 4] = value.to_bytes(4, 'big')


def parse_utf8(data: bytes) -> tuple[dict[int, str], int]:
    count = read_u2(data, 8)
    index = 1
    offset = 10
    utf8: dict[int, str] = {}
    while index < count:
        tag = data[offset]
        offset += 1
        if tag == 1:
            length = read_u2(data, offset)
            offset += 2
            utf8[index] = data[offset:offset + length].decode('ascii')
            offset += length
        elif tag in (3, 4):
            offset += 4
        elif tag in (5, 6):
            offset += 8
            index += 1
        elif tag in (7, 8, 16, 19, 20):
            offset += 2
        elif tag in (9, 10, 11, 12, 17, 18):
            offset += 4
        elif tag == 15:
            offset += 3
        else:
            raise ValueError(f'unsupported constant-pool tag {tag} at index {index}')
        index += 1
    return utf8, offset


def code_attribute(data: bytes | bytearray, target_name: str) -> tuple[int, int, int, int]:
    """Return the Code attribute length field, code-length field, code start, and length."""
    utf8, offset = parse_utf8(data)
    offset += 6  # access_flags, this_class, super_class
    interface_count = read_u2(data, offset)
    offset += 2 + 2 * interface_count

    def skip_members(at: int, count: int) -> int:
        for _ in range(count):
            at += 6  # access, name, descriptor
            attributes = read_u2(data, at)
            at += 2
            for _ in range(attributes):
                length = read_u4(data, at + 2)
                at += 6 + length
        return at

    fields = read_u2(data, offset)
    offset = skip_members(offset + 2, fields)
    methods = read_u2(data, offset)
    offset += 2
    for _ in range(methods):
        name = utf8[read_u2(data, offset + 2)]
        offset += 6
        attributes = read_u2(data, offset)
        offset += 2
        for _ in range(attributes):
            attribute_name = utf8[read_u2(data, offset)]
            attribute_length_offset = offset + 2
            length = read_u4(data, attribute_length_offset)
            info = offset + 6
            if name == target_name and attribute_name == 'Code':
                code_length_offset = info + 4
                code_length = read_u4(data, code_length_offset)
                return attribute_length_offset, code_length_offset, info + 8, code_length
            offset += 6 + length
    raise ValueError(f'no Code attribute found for method {target_name}')


def patch_method_code(path: Path, target_name: str, expected: bytes, replacement: bytes) -> dict[str, object]:
    data = bytearray(path.read_bytes())
    attribute_length_offset, code_length_offset, code_start, code_length = code_attribute(data, target_name)
    old_code = bytes(data[code_start:code_start + code_length])
    if old_code.count(expected) != 1:
        raise ValueError(f'{target_name}: expected one {expected.hex()} in Code {old_code.hex()}')
    patched_code = old_code.replace(expected, replacement)
    updated = data[:code_start] + patched_code + data[code_start + code_length:]
    size_delta = len(patched_code) - code_length
    write_u4(updated, code_length_offset, len(patched_code))
    write_u4(updated, attribute_length_offset, read_u4(data, attribute_length_offset) + size_delta)
    path.write_bytes(updated)
    return {
        'method': target_name,
        'old_code_hex': old_code.hex(),
        'new_code_hex': patched_code.hex(),
        'old_code_length': code_length,
        'new_code_length': len(patched_code),
        'replacement_old_hex': expected.hex(),
        'replacement_new_hex': replacement.hex(),
    }


def patch_descriptor(path: Path, old: bytes, new: bytes) -> dict[str, str | int]:
    data = path.read_bytes()
    if len(old) != len(new) or data.count(old) != 1:
        raise ValueError(f'expected one same-width descriptor {old!r} in class file')
    path.write_bytes(data.replace(old, new))
    return {'old': old.decode('ascii'), 'new': new.decode('ascii'), 'offset': data.index(old)}


def decompile(kind: str, class_file: Path, out: Path, base: Path, results: dict[str, object]) -> Path:
    out.mkdir(parents=True, exist_ok=True)
    if kind == 'jadx':
        rc = run(['jadx', '--no-res', '-d', str(out), str(class_file)], EVIDENCE / f'{base.name}-jadx.log')
    else:
        command = [str(CLI), 'class-source', '--input', str(class_file), '--class', 'BoundaryProbe',
                   '--policy', 'single-class', '--release', '8', '--format', 'text']
        result = subprocess.run(command, capture_output=True, text=True, timeout=60)
        (out / 'BoundaryProbe.java').write_text(result.stdout)
        (EVIDENCE / f'{base.name}-jarde.log').write_text(result.stderr)
        rc = result.returncode
    results[f'{base.name}_{kind}_decompile_rc'] = rc
    candidates = list(out.rglob('BoundaryProbe.java'))
    if len(candidates) != 1:
        raise ValueError(f'{kind}: expected one BoundaryProbe.java, found {candidates}')
    generated = candidates[0]
    shutil.copy2(generated, EVIDENCE / f'{base.name}-{kind}.java.txt')
    return generated


def compile_recovered(generated: Path, work: Path, mode: str, label: str,
                      results: dict[str, object]) -> None:
    compile_dir = work / f'{label}-classes'
    compile_dir.mkdir(parents=True, exist_ok=True)
    runner_copy = compile_dir / 'BoundaryRunner.java'
    shutil.copy2(RUNNER_SOURCE, runner_copy)
    recovered = compile_dir / 'BoundaryProbe.java'
    shutil.copy2(generated, recovered)
    javac_log = EVIDENCE / f'{label}-javac.log'
    rc = run(['javac', '--release', '8', '-g:none', '-d', str(compile_dir),
              str(recovered), str(runner_copy)], javac_log)
    results[f'{label}_javac_rc'] = rc
    if rc == 0:
        package = next((line.split()[1].rstrip(';') for line in generated.read_text().splitlines()
                        if line.strip().startswith('package ')), '')
        binary_name = f'{package}.BoundaryProbe' if package else 'BoundaryProbe'
        run_rc = run(['java', '-Xverify:all', '-cp', str(compile_dir), 'BoundaryRunner', mode, binary_name],
                     EVIDENCE / f'{label}-run.txt')
        results[f'{label}_run_rc'] = run_rc


def main() -> None:
    if WORK.exists():
        shutil.rmtree(WORK)
    WORK.mkdir(parents=True)
    results: dict[str, object] = {
        'java_version': subprocess.run(['java', '-version'], capture_output=True, text=True).stderr.strip(),
        'javac_version': subprocess.run(['javac', '-version'], capture_output=True, text=True).stdout.strip(),
        'jadx_version_rc': run(['jadx', '--version'], EVIDENCE / 'jadx-version.log'),
        'frozen_cli_sha256': sha256(CLI),
        'scope': 'Two Z/I descriptor orderings, an integer 0/1 control, overwritten local, and one dup/call/pop delayed-consumer Code variant.',
    }
    if results['frozen_cli_sha256'] != '88f2e7aa9b5b8172da02f5af5d4d2c029e72a52d9b3298b46b82774e0bd5fdfd':
        raise ValueError('the frozen CLI hash does not match the requested root baseline')

    source_dir = WORK / 'source'
    source_dir.mkdir()
    original_class = source_dir / 'BoundaryProbe.class'
    results['source_javac_rc'] = run(['javac', '--release', '8', '-g:none', '-d', str(source_dir),
                                      str(PROBE_SOURCE), str(RUNNER_SOURCE)], EVIDENCE / 'source-javac.log')
    if results['source_javac_rc'] != 0:
        raise ValueError('audit source did not compile')
    results['source_class_sha256'] = sha256(original_class)
    results['source_class_length'] = original_class.stat().st_size
    results['source_java_verify_rc'] = run(['java', '-Xverify:all', '-cp', str(source_dir),
                                            'BoundaryRunner', 'source'], EVIDENCE / 'source-run.txt')
    results['source_javap_rc'] = run(['javap', '-v', '-c', '-p', str(original_class)],
                                     EVIDENCE / 'source-javap.txt')

    # A Java caller cannot declare these descriptor variants. They remain legal class files because
    # boolean and int parameters share the verifier's integral computational type.
    patched_dir = WORK / 'patched'
    patched_dir.mkdir()
    patched_class = patched_dir / 'BoundaryProbe.class'
    shutil.copy2(original_class, patched_class)
    descriptors = [
        patch_descriptor(patched_class, b'(IIJ)I', b'(ZIJ)I'),
        patch_descriptor(patched_class, b'(IIB)I', b'(IZB)I'),
    ]
    # The method reference seeded by seedObserve is the only `observe:(I)I` invocation.
    cp_count = read_u2(patched_class.read_bytes(), 8)
    raw = patched_class.read_bytes()
    offset = 10
    class_indexes: dict[int, int] = {}
    name_type_indexes: dict[int, tuple[int, int]] = {}
    member_refs: dict[int, tuple[int, int]] = {}
    utf8, _ = parse_utf8(raw)
    index = 1
    while index < cp_count:
        tag = raw[offset]
        offset += 1
        if tag == 1:
            length = read_u2(raw, offset)
            offset += 2 + length
        elif tag in (3, 4):
            offset += 4
        elif tag in (5, 6):
            offset += 8
            index += 1
        elif tag in (7, 8, 16, 19, 20):
            class_indexes[index] = read_u2(raw, offset)
            offset += 2
        elif tag in (9, 10, 11):
            member_refs[index] = (read_u2(raw, offset), read_u2(raw, offset + 2))
            offset += 4
        elif tag in (12, 17, 18):
            name_type_indexes[index] = (read_u2(raw, offset), read_u2(raw, offset + 2))
            offset += 4
        elif tag == 15:
            offset += 3
        else:
            raise ValueError(f'unsupported constant-pool tag {tag}')
        index += 1
    observe_ref = None
    for cp_index, (owner_index, name_type_index) in member_refs.items():
        pair = name_type_indexes.get(name_type_index)
        if pair and utf8.get(pair[0]) == 'observe' and utf8.get(pair[1]) == '(I)I':
            owner_name_index = class_indexes.get(owner_index)
            if owner_name_index is not None and utf8.get(owner_name_index) == 'BoundaryProbe':
                observe_ref = cp_index
                break
    if observe_ref is None:
        raise ValueError('did not find seeded BoundaryProbe.observe:(I)I Methodref')
    old_observe_code = bytes([0x7e, 0x3d, 0x1c, 0xb8, observe_ref >> 8,
                              observe_ref & 0xff, 0x57, 0x1c, 0xac])
    new_observe_code = bytes([0x7e, 0x59, 0xb8, observe_ref >> 8,
                              observe_ref & 0xff, 0x57, 0xac])
    code_patch = patch_method_code(patched_class, 'observeOnce', old_observe_code, new_observe_code)
    results['descriptor_patches'] = descriptors
    results['observe_methodref_cp_index'] = observe_ref
    results['dup_pop_code_patch'] = code_patch
    results['patched_class_sha256'] = sha256(patched_class)
    results['patched_class_length'] = patched_class.stat().st_size
    results['patched_runner_javac_rc'] = run(['javac', '--release', '8', '-g:none', '-cp', str(patched_dir),
                                               '-d', str(patched_dir), str(RUNNER_SOURCE)],
                                              EVIDENCE / 'patched-runner-javac.log')
    results['patched_java_verify_rc'] = run(['java', '-Xverify:all', '-cp', str(patched_dir),
                                             'BoundaryRunner', 'patched', 'BoundaryProbe'],
                                            EVIDENCE / 'patched-run.txt')
    results['patched_javap_rc'] = run(['javap', '-v', '-c', '-p', str(patched_class)],
                                      EVIDENCE / 'patched-javap.txt')

    # Compare complete decompiler output classes with the same runner. The patched class uses the
    # descriptor-driven reflective call mode; no decompiled body is manually edited.
    for kind in ('jadx', 'jarde'):
        generated = decompile(kind, original_class, WORK / f'{kind}-source', WORK / 'source', results)
        compile_recovered(generated, WORK, 'source', f'source-{kind}', results)
    for kind in ('jadx', 'jarde'):
        generated = decompile(kind, patched_class, WORK / f'patched-{kind}-source',
                              WORK / 'patched', results)
        compile_recovered(generated, WORK, 'patched', f'patched-{kind}', results)

    naive_source = '''public final class NaiveMixed {
    static int left(boolean value, int other) { return value & other; }
    static int right(int value, boolean other) { return value | other; }
}
'''
    (EVIDENCE / 'naive-mixed.java.txt').write_text(naive_source)
    naive_dir = WORK / 'naive'
    naive_dir.mkdir()
    (naive_dir / 'NaiveMixed.java').write_text(naive_source)
    results['naive_mixed_javac_rc'] = run(['javac', '--release', '8', '-d', str(naive_dir),
                                           str(naive_dir / 'NaiveMixed.java')],
                                          EVIDENCE / 'naive-mixed-javac.log')

    source_output = (EVIDENCE / 'source-run.txt').read_text().splitlines()
    jadx_output = (EVIDENCE / 'source-jadx-run.txt').read_text().splitlines()
    patched_output = (EVIDENCE / 'patched-run.txt').read_text().splitlines()
    shared_prefix = ('pure-int-01=', 'overwritten-local=', 'dup-call-pop=')
    source_shared = [line for line in source_output if line.startswith(shared_prefix)]
    patched_shared = [line for line in patched_output if line.startswith(shared_prefix)]
    source_trace = next(line for line in source_output if line.startswith('trace='))
    patched_trace = next(line for line in patched_output if line.startswith('trace='))
    results['source_vs_jadx_runtime_equal'] = source_output == jadx_output
    results['source_vs_patched_shared_semantics_equal'] = (
        source_shared == patched_shared and source_trace == patched_trace
    )
    results['frozen_cli_sha256_after'] = sha256(CLI)

    summary_path = EVIDENCE / 'summary.json'
    summary_path.write_text(json.dumps(results, indent=2, ensure_ascii=False) + '\n')
    print(json.dumps(results, indent=2, ensure_ascii=False))


if __name__ == '__main__':
    main()
