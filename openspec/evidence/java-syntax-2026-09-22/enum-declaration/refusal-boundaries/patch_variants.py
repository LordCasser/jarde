#!/usr/bin/env python3
"""Make two one-change, JVM-valid negative-boundary patches of compiled Measure.class."""
from __future__ import annotations

import hashlib
import json
import struct
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
SOURCE_CLASS = Path(sys.argv[1]) if len(sys.argv) > 1 else HERE / 'classes-original' / 'Measure.class'
if not SOURCE_CLASS.is_absolute():
    SOURCE_CLASS = HERE / SOURCE_CLASS


def u2(data: bytes | bytearray, offset: int) -> int:
    return struct.unpack_from('>H', data, offset)[0]


def u4(data: bytes | bytearray, offset: int) -> int:
    return struct.unpack_from('>I', data, offset)[0]


def parse_pool(data: bytes | bytearray):
    count = u2(data, 8)
    pool: list[tuple[int, object] | None] = [None] * count
    offsets: dict[int, tuple[int, int]] = {}
    offset = 10
    index = 1
    while index < count:
        start = offset
        tag = data[offset]
        offset += 1
        if tag == 1:
            size = u2(data, offset)
            offset += 2
            raw = bytes(data[offset:offset + size])
            offset += size
            value = raw.decode('utf-8')
        elif tag in (3, 4):
            value = bytes(data[offset:offset + 4])
            offset += 4
        elif tag in (5, 6):
            value = bytes(data[offset:offset + 8])
            offset += 8
            pool[index] = (tag, value)
            offsets[index] = (start, offset - start)
            index += 1
            continue
        elif tag in (7, 8, 16, 19, 20):
            value = u2(data, offset)
            offset += 2
        elif tag in (9, 10, 11, 12, 17, 18):
            value = (u2(data, offset), u2(data, offset + 2))
            offset += 4
        elif tag == 15:
            value = (data[offset], u2(data, offset + 1))
            offset += 3
        else:
            raise AssertionError(f'unsupported constant-pool tag {tag}')
        pool[index] = (tag, value)
        offsets[index] = (start, offset - start)
        index += 1
    return pool, offsets, offset


def utf8(pool, index: int) -> str:
    entry = pool[index]
    assert entry and entry[0] == 1
    return entry[1]


def members(data: bytes | bytearray, pool, cp_end: int):
    offset = cp_end + 6
    interfaces = u2(data, offset)
    offset += 2 + 2 * interfaces
    tables = []
    for table_name in ('fields', 'methods'):
        count = u2(data, offset)
        offset += 2
        declarations = []
        for _ in range(count):
            flags = u2(data, offset)
            name = utf8(pool, u2(data, offset + 2))
            desc = utf8(pool, u2(data, offset + 4))
            attributes = u2(data, offset + 6)
            declarations.append((flags, name, desc))
            offset += 8
            for _ in range(attributes):
                length = u4(data, offset + 2)
                offset += 6 + length
        tables.append((table_name, declarations))
    return tables


def clinit_code(data: bytes | bytearray, pool, cp_end: int):
    offset = cp_end + 6
    offset += 2 + 2 * u2(data, offset)
    field_count = u2(data, offset)
    offset += 2
    for _ in range(field_count):
        attributes = u2(data, offset + 6)
        offset += 8
        for _ in range(attributes):
            offset += 6 + u4(data, offset + 2)
    method_count = u2(data, offset)
    offset += 2
    for _ in range(method_count):
        name = utf8(pool, u2(data, offset + 2))
        descriptor = utf8(pool, u2(data, offset + 4))
        attributes = u2(data, offset + 6)
        offset += 8
        for _ in range(attributes):
            attr_name = utf8(pool, u2(data, offset))
            length = u4(data, offset + 2)
            info = offset + 6
            if name == '<clinit>' and descriptor == '()V' and attr_name == 'Code':
                code_length = u4(data, info + 4)
                code_start = info + 8
                return code_start, code_length
            offset = info + length
    raise AssertionError('missing <clinit>()V Code')


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def main() -> None:
    original = SOURCE_CLASS.read_bytes()
    pool, offsets, cp_end = parse_pool(original)
    assert u4(original, 0) == 0xCAFEBABE
    assert u2(original, 6) == 52, 'expected Java 8 class-file major version'
    original_tables = members(original, pool, cp_end)

    # Change only the CONSTANT_String reference used by ldc "LOW". Append a fresh Utf8 so the
    # field name LOW and every member-table index remain untouched.
    low_utf8 = [i for i, entry in enumerate(pool) if entry and entry[0] == 1 and entry[1] == 'LOW']
    low_strings = [i for i, entry in enumerate(pool) if entry and entry[0] == 8 and entry[1] in low_utf8]
    assert len(low_utf8) == 1 and len(low_strings) == 1
    new_index = len(pool)
    alias_utf8 = b'\x01' + struct.pack('>H', len(b'ALIAS')) + b'ALIAS'
    name_patched = bytearray(original)
    string_offset, _ = offsets[low_strings[0]]
    name_patched[string_offset + 1:string_offset + 3] = struct.pack('>H', new_index)
    name_patched[8:10] = struct.pack('>H', u2(original, 8) + 1)
    name_patched[cp_end:cp_end] = alias_utf8
    name_patched = bytes(name_patched)
    name_pool, _, name_cp_end = parse_pool(name_patched)
    assert members(name_patched, name_pool, name_cp_end) == original_tables

    # Change only the second constant's ordinal constructor argument from iconst_1 to iconst_2.
    # The runner source is intentionally outside this class patch; the constructor receives a
    # different ordinal while the fields and methods stay identical.
    high_strings = [i for i, entry in enumerate(pool)
                    if entry and entry[0] == 8 and utf8(pool, entry[1]) == 'HIGH']
    assert len(high_strings) == 1 and high_strings[0] < 256
    code_start, code_length = clinit_code(original, pool, cp_end)
    code = original[code_start:code_start + code_length]
    marker = bytes((0x12, high_strings[0], 0x04, 0x08, 0xB7))
    assert code.count(marker) == 1, f'expected one HIGH/name, ordinal-1, units-5 constructor sequence, found {code.count(marker)}'
    patched_code = code.replace(marker, bytes((0x12, high_strings[0], 0x05, 0x08, 0xB7)), 1)
    ordinal_patched = bytearray(original)
    ordinal_patched[code_start:code_start + code_length] = patched_code
    ordinal_patched = bytes(ordinal_patched)
    ordinal_pool, _, ordinal_cp_end = parse_pool(ordinal_patched)
    assert members(ordinal_patched, ordinal_pool, ordinal_cp_end) == original_tables

    outputs = {'name-alias': name_patched, 'ordinal-two': ordinal_patched}
    manifest = {
        'input': str(SOURCE_CLASS.relative_to(HERE)),
        'class_major': 52,
        'original_sha256': sha(original),
        'member_tables_unchanged': True,
        'patches': {},
    }
    for name, patched in outputs.items():
        variant_dir = HERE / f'patched-{name}'
        variant_dir.mkdir(parents=True, exist_ok=True)
        path = variant_dir / 'Measure.class'
        path.write_bytes(patched)
        manifest['patches'][name] = {
            'path': str(path.relative_to(HERE)),
            'sha256': sha(patched),
            'length': len(patched),
            'edit': ('retarget the CONSTANT_String used for constructor name "LOW" to a new Utf8 "ALIAS"'
                     if name == 'name-alias' else 'change HIGH constructor ordinal operand from iconst_1 to iconst_2'),
            'member_tables_unchanged': members(patched, parse_pool(patched)[0], parse_pool(patched)[2]) == original_tables,
        }
    (HERE / 'patch-manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')


if __name__ == '__main__':
    main()
