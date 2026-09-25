#!/usr/bin/env python3
"""Create verifier-safe, single-method enum helper mutations without changing member tables."""
from __future__ import annotations

import hashlib
import json
import struct
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
SOURCE_CLASS = Path(sys.argv[1]) if len(sys.argv) > 1 else HERE / 'generated' / 'source-baseline' / 'Measure.class'
OUTPUT_ROOT = Path(sys.argv[2]) if len(sys.argv) > 2 else HERE / 'generated'


def u2(data: bytes | bytearray, offset: int) -> int:
    return struct.unpack_from('>H', data, offset)[0]


def u4(data: bytes | bytearray, offset: int) -> int:
    return struct.unpack_from('>I', data, offset)[0]


def parse_pool(data: bytes | bytearray):
    count = u2(data, 8)
    pool: list[tuple[int, object] | None] = [None] * count
    offset = 10
    index = 1
    while index < count:
        tag = data[offset]
        offset += 1
        if tag == 1:
            size = u2(data, offset)
            offset += 2
            value = bytes(data[offset:offset + size]).decode('utf-8')
            offset += size
        elif tag in (3, 4):
            value = bytes(data[offset:offset + 4])
            offset += 4
        elif tag in (5, 6):
            value = bytes(data[offset:offset + 8])
            offset += 8
            pool[index] = (tag, value)
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
        index += 1
    return pool, offset


def utf8(pool, index: int) -> str:
    entry = pool[index]
    assert entry and entry[0] == 1
    return entry[1]


def class_name(pool, index: int) -> str:
    entry = pool[index]
    assert entry and entry[0] == 7
    return utf8(pool, entry[1])


def name_and_type(pool, index: int) -> tuple[str, str]:
    entry = pool[index]
    assert entry and entry[0] == 12
    name_index, descriptor_index = entry[1]
    return utf8(pool, name_index), utf8(pool, descriptor_index)


def field_reference(pool, index: int) -> tuple[str, str, str]:
    entry = pool[index]
    assert entry and entry[0] == 9
    owner_index, name_type_index = entry[1]
    name, descriptor = name_and_type(pool, name_type_index)
    return class_name(pool, owner_index), name, descriptor


def member_tables(data: bytes | bytearray, pool, cp_end: int):
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
            descriptor = utf8(pool, u2(data, offset + 4))
            attributes = u2(data, offset + 6)
            declarations.append((flags, name, descriptor))
            offset += 8
            for _ in range(attributes):
                length = u4(data, offset + 2)
                offset += 6 + length
        tables.append((table_name, declarations))
    return tables


def method_code(data: bytes | bytearray, pool, cp_end: int, wanted: tuple[str, str]):
    offset = cp_end + 6
    offset += 2 + 2 * u2(data, offset)
    fields = u2(data, offset)
    offset += 2
    for _ in range(fields):
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
            attribute_name = utf8(pool, u2(data, offset))
            length = u4(data, offset + 2)
            info = offset + 6
            if (name, descriptor) == wanted and attribute_name == 'Code':
                code_length = u4(data, info + 4)
                return info + 8, code_length
            offset = info + length
    raise AssertionError(f'missing Code for {wanted}')


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def table_report(tables):
    return [
        {
            'table': name,
            'declarations': [
                {'access_flags': flags, 'name': member, 'descriptor': descriptor}
                for flags, member, descriptor in declarations
            ],
        }
        for name, declarations in tables
    ]


def main() -> None:
    original = SOURCE_CLASS.read_bytes()
    pool, cp_end = parse_pool(original)
    assert u4(original, 0) == 0xCAFEBABE
    assert u2(original, 6) == 52, 'expected Java 8 class-file major version'
    original_tables = member_tables(original, pool, cp_end)

    values_start, values_length = method_code(original, pool, cp_end, ('$values', '()[LMeasure;'))
    values_code = original[values_start:values_start + values_length]
    assert values_length == 17 and values_code == bytes((
        0x05, 0xbd, values_code[2], values_code[3], 0x59, 0x03, 0xb2,
        values_code[7], values_code[8], 0x53, 0x59, 0x04, 0xb2,
        values_code[13], values_code[14], 0x53, 0xb0,
    )), 'javac $values shape differs from the audited Java 8 helper'
    low_ref = u2(values_code, 7)
    high_ref = u2(values_code, 13)
    assert field_reference(pool, low_ref) == ('Measure', 'LOW', 'LMeasure;')
    assert field_reference(pool, high_ref) == ('Measure', 'HIGH', 'LMeasure;')

    order = bytearray(original)
    order[values_start + 7:values_start + 9] = struct.pack('>H', high_ref)
    order[values_start + 13:values_start + 15] = struct.pack('>H', low_ref)

    value_of_start, value_of_length = method_code(
        original, pool, cp_end, ('valueOf', '(Ljava/lang/String;)LMeasure;')
    )
    value_of_code = original[value_of_start:value_of_start + value_of_length]
    assert value_of_length == 10 and value_of_code[0] == 0x12 and value_of_code[2:4] == b'\x2a\xb8'
    parameter = bytearray(original)
    parameter[value_of_start + 2] = 0x01  # aconst_null in place of aload_0

    outputs = {'values-order': bytes(order), 'valueof-null-argument': bytes(parameter)}
    tables_json = table_report(original_tables)
    tables_digest = hashlib.sha256(
        json.dumps(tables_json, sort_keys=True, separators=(',', ':')).encode()
    ).hexdigest()
    manifest = {
        'input': str(SOURCE_CLASS.relative_to(HERE)) if SOURCE_CLASS.is_relative_to(HERE) else str(SOURCE_CLASS),
        'class_major': 52,
        'original_sha256': sha(original),
        'field_and_method_tables_unchanged': True,
        'member_table_sha256': tables_digest,
        'member_tables': tables_json,
        'patches': {},
    }
    for name, patched in outputs.items():
        patched_pool, patched_cp_end = parse_pool(patched)
        assert member_tables(patched, patched_pool, patched_cp_end) == original_tables
        changed_offsets = [i for i, (old, new) in enumerate(zip(original, patched)) if old != new]
        if name == 'values-order':
            expected = [
                values_start + bci_offset + byte_offset
                for bci_offset, old_ref, new_ref in ((7, low_ref, high_ref), (13, high_ref, low_ref))
                for byte_offset, (old, new) in enumerate(zip(struct.pack('>H', old_ref), struct.pack('>H', new_ref)))
                if old != new
            ]
            expected_bcis = [6, 12]
            edit = 'swap only the LOW/HIGH getstatic operands in $values()'
        else:
            expected = [value_of_start + 2]
            expected_bcis = [2]
            edit = 'replace valueOf(String) argument load with aconst_null'
        assert changed_offsets == expected, (name, changed_offsets, expected)
        variant_dir = OUTPUT_ROOT / f'patched-{name}'
        variant_dir.mkdir(parents=True, exist_ok=True)
        path = variant_dir / 'Measure.class'
        path.write_bytes(patched)
        manifest['patches'][name] = {
            'path': str(path.relative_to(HERE)),
            'sha256': sha(patched),
            'length': len(patched),
            'changed_code_bcis': expected_bcis,
            'edit': edit,
            'field_and_method_tables_unchanged': True,
            'member_table_sha256': tables_digest,
            'changed_offsets_confined_to_target_method_code': True,
        }
    (HERE / 'patch-manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')


if __name__ == '__main__':
    main()
