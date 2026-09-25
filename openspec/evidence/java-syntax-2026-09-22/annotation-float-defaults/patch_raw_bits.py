#!/usr/bin/env python3
"""Patch one AnnotationDefault float/double CONSTANT payload at a time."""
from __future__ import annotations

import hashlib
import json
import struct
import shutil
from pathlib import Path

HERE = Path(__file__).resolve().parent
BASE = HERE / 'classes-original' / 'FloatDefaults.class'
VARIANTS = {
    'positive-infinity-to-negative-qnan': ('positiveInfinity', b'F', 0x7f800000, 0xffc00000, 4),
    'canonical-nan-to-payload-nan': ('canonicalNaN', b'D', 0x7ff8000000000000, 0x7ff8000000000001, 8),
}


def u2(data: bytes | bytearray, off: int) -> int:
    return struct.unpack_from('>H', data, off)[0]


def u4(data: bytes | bytearray, off: int) -> int:
    return struct.unpack_from('>I', data, off)[0]


def parse_pool(data: bytes | bytearray):
    count = u2(data, 8)
    pool: list[tuple[int, object] | None] = [None] * count
    payload_offsets: dict[int, int] = {}
    offset = 10
    index = 1
    while index < count:
        start = offset
        tag = data[offset]
        offset += 1
        if tag == 1:
            length = u2(data, offset)
            offset += 2
            raw = bytes(data[offset:offset + length])
            offset += length
            value = raw.decode('utf-8')
        elif tag in (3, 4):
            value = bytes(data[offset:offset + 4])
            payload_offsets[index] = offset
            offset += 4
        elif tag in (5, 6):
            value = bytes(data[offset:offset + 8])
            payload_offsets[index] = offset
            offset += 8
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
        if tag in (5, 6):
            index += 1
            if index < count:
                pool[index] = None
        index += 1
    return pool, payload_offsets, offset


def utf8(pool, index: int) -> str:
    entry = pool[index]
    assert entry and entry[0] == 1
    return entry[1]


def member_table(data: bytes | bytearray, pool, cp_end: int):
    offset = cp_end + 6
    interface_count = u2(data, offset)
    offset += 2 + 2 * interface_count
    tables = []
    for kind in ('fields', 'methods'):
        count = u2(data, offset)
        offset += 2
        members = []
        for _ in range(count):
            flags = u2(data, offset)
            name = utf8(pool, u2(data, offset + 2))
            descriptor = utf8(pool, u2(data, offset + 4))
            attr_count = u2(data, offset + 6)
            members.append((flags, name, descriptor))
            offset += 8
            for _ in range(attr_count):
                offset += 6 + u4(data, offset + 2)
        tables.append((kind, members))
    return tables


def annotation_default_indices(data: bytes | bytearray, pool, cp_end: int):
    offset = cp_end + 6
    interface_count = u2(data, offset)
    offset += 2 + 2 * interface_count
    field_count = u2(data, offset)
    offset += 2
    for _ in range(field_count):
        attr_count = u2(data, offset + 6)
        offset += 8
        for _ in range(attr_count):
            offset += 6 + u4(data, offset + 2)
    method_count = u2(data, offset)
    offset += 2
    found = {}
    for _ in range(method_count):
        name = utf8(pool, u2(data, offset + 2))
        descriptor = utf8(pool, u2(data, offset + 4))
        attr_count = u2(data, offset + 6)
        offset += 8
        for _ in range(attr_count):
            attr_name = utf8(pool, u2(data, offset))
            length = u4(data, offset + 2)
            info = offset + 6
            if attr_name == 'AnnotationDefault':
                assert length == 3
                tag = bytes(data[info:info + 1])
                index = u2(data, info + 1)
                found[name] = (descriptor, tag, index)
            offset = info + length
    return found


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def main() -> None:
    original = BASE.read_bytes()
    assert len(original) == 293
    assert u4(original, 0) == 0xcafebabe and u2(original, 6) == 52
    pool, cp_payloads, cp_end = parse_pool(original)
    original_tables = member_table(original, pool, cp_end)
    defaults = annotation_default_indices(original, pool, cp_end)
    manifest = {
        'baseline': {'path': 'classes-original/FloatDefaults.class', 'length': len(original), 'sha256': sha(original)},
        'class_major': u2(original, 6),
        'member_tables_unchanged': True,
        'patches': {},
    }
    for dirname, (method, expected_tag, old_bits, new_bits, size) in VARIANTS.items():
        descriptor, tag, cp_index = defaults[method]
        assert tag == expected_tag and descriptor in ('()F', '()D')
        entry = pool[cp_index]
        expected_cp_tag = 4 if size == 4 else 6
        assert entry and entry[0] == expected_cp_tag
        old_raw = int.from_bytes(entry[1], 'big')
        assert old_raw == old_bits, (method, hex(old_raw), hex(old_bits))
        patched = bytearray(original)
        data_offset = cp_payloads[cp_index]
        patched[data_offset:data_offset + size] = new_bits.to_bytes(size, 'big')
        patched_bytes = bytes(patched)
        changed = [i for i, (a, b) in enumerate(zip(original, patched_bytes)) if a != b]
        assert changed and all(data_offset <= i < data_offset + size for i in changed), (method, changed)
        assert int.from_bytes(patched_bytes[data_offset:data_offset + size], 'big') == new_bits
        patched_pool, _, patched_cp_end = parse_pool(patched_bytes)
        assert u2(patched_bytes, 6) == 52
        assert member_table(patched_bytes, patched_pool, patched_cp_end) == original_tables
        assert cp_end == patched_cp_end and original[cp_end:] == patched_bytes[patched_cp_end:]
        out_dir = HERE / dirname
        shutil.rmtree(out_dir, ignore_errors=True)
        out_dir.mkdir()
        (out_dir / 'FloatDefaults.class').write_bytes(patched_bytes)
        manifest['patches'][dirname] = {
            'path': f'{dirname}/FloatDefaults.class',
            'annotation_element': method,
            'element_tag': tag.decode('ascii'),
            'constant_pool_index': cp_index,
            'constant_pool_payload_offset': data_offset,
            'old_raw_bits': f'{old_bits:0{size * 2}x}',
            'new_raw_bits': f'{new_bits:0{size * 2}x}',
            'changed_byte_offsets': changed,
            'length': len(patched_bytes),
            'sha256': sha(patched_bytes),
            'class_major': u2(patched_bytes, 6),
            'member_tables_unchanged': True,
            'member_table_bytes_unchanged': True,
            'non_target_bytes_unchanged': True,
        }
    (HERE / 'patch-manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')


if __name__ == '__main__':
    main()
