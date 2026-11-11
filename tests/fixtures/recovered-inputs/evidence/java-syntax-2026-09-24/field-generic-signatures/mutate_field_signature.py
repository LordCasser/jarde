#!/usr/bin/env python3
"""Replace one field's Signature Utf8 value while leaving its descriptor untouched."""

import struct
import sys
from pathlib import Path


def u2(data: bytes, offset: int) -> int:
    return struct.unpack_from(">H", data, offset)[0]


def u4(data: bytes, offset: int) -> int:
    return struct.unpack_from(">I", data, offset)[0]


def main() -> None:
    if len(sys.argv) != 6:
        raise SystemExit(
            "usage: mutate_field_signature.py INPUT OUTPUT FIELD OLD_SIGNATURE NEW_SIGNATURE"
        )
    source_path, output_path, field_name, old_sig, new_sig = sys.argv[1:]
    data = Path(source_path).read_bytes()
    if data[:4] != b"\xca\xfe\xba\xbe":
        raise SystemExit("not a class file")

    cp_count = u2(data, 8)
    cp: dict[int, tuple[int, int, bytes]] = {}
    cursor = 10
    while len(cp) < cp_count - 1:
        index = len(cp) + 1
        tag = data[cursor]
        start = cursor
        cursor += 1
        if tag == 1:
            size = u2(data, cursor)
            cursor += 2
            raw = data[cursor : cursor + size]
            cp[index] = (tag, start, raw)
            cursor += size
        elif tag in (3, 4, 9, 10, 11, 12, 17, 18):
            cursor += 4
        elif tag in (5, 6):
            cursor += 8
            cp[index] = (tag, start, b"")
            cp[index + 1] = (0, cursor, b"")
        elif tag in (7, 8, 16, 19, 20):
            cursor += 2
        elif tag == 15:
            cursor += 3
        else:
            raise SystemExit(f"unsupported constant-pool tag {tag}")
        if tag not in (5, 6) and index not in cp:
            cp[index] = (tag, start, b"")

    def utf8(index: int) -> str:
        tag, _, raw = cp[index]
        if tag != 1:
            raise SystemExit(f"constant pool entry {index} is not Utf8")
        return raw.decode("utf-8")

    class_header = cursor
    interfaces_count_offset = class_header + 6
    interfaces_count = u2(data, interfaces_count_offset)
    fields_count_offset = interfaces_count_offset + 2 + 2 * interfaces_count
    fields_count = u2(data, fields_count_offset)
    field_cursor = fields_count_offset + 2
    matched: list[int] = []
    for _ in range(fields_count):
        name_index = u2(data, field_cursor + 2)
        attr_count = u2(data, field_cursor + 6)
        field_cursor += 8
        is_target = utf8(name_index) == field_name
        for _ in range(attr_count):
            attr_name = u2(data, field_cursor)
            attr_length = u4(data, field_cursor + 2)
            attr_body = field_cursor + 6
            if is_target and utf8(attr_name) == "Signature":
                if attr_length != 2:
                    raise SystemExit("invalid Signature attribute length")
                matched.append(u2(data, attr_body))
            field_cursor = attr_body + attr_length
    if len(matched) != 1:
        raise SystemExit(f"expected one Signature on field {field_name}, found {len(matched)}")
    signature_index = matched[0]
    tag, cp_start, raw = cp[signature_index]
    if tag != 1 or raw.decode("utf-8") != old_sig:
        raise SystemExit("field Signature did not match the expected old value")

    replacement = new_sig.encode("utf-8")
    encoded_entry = b"\x01" + struct.pack(">H", len(replacement)) + replacement
    old_end = cp_start + 3 + len(raw)
    output = data[:cp_start] + encoded_entry + data[old_end:]
    Path(output_path).write_bytes(output)


if __name__ == "__main__":
    main()
