#!/usr/bin/env python3
"""Patch only the original 45.3 method Code payload into two private CFG controls."""
from pathlib import Path
import struct

ROOT = Path(__file__).parent
SOURCE = ROOT / "original/loops/TestLoopRestore2.class"
OUTPUTS = {
    "LoopRestoreGotoW.class": bytes.fromhex(
        "033e110000990011c80000000511000199fffda70003840301b1"
    ),
    "LoopRestoreEffectPrefix.class": bytes.fromhex(
        "033e110000990012840301a7000311000199fffda70003840301b1"
    ),
}
ORIGINAL_CODE = bytes.fromhex("033e11000099000fa7000311000199fffda70003840301b1")


def replace_code(class_bytes: bytes, replacement: bytes) -> bytes:
    data = bytearray(class_bytes)
    cp_count = struct.unpack_from(">H", data, 8)[0]
    utf8: dict[int, str] = {}
    offset, index = 10, 1
    while index < cp_count:
        tag = data[offset]
        offset += 1
        if tag == 1:
            length = struct.unpack_from(">H", data, offset)[0]
            utf8[index] = data[offset + 2 : offset + 2 + length].decode("utf-8")
            offset += 2 + length
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
            raise ValueError(f"unknown constant-pool tag {tag}")
        index += 1

    offset += 6  # access, this, super
    interface_count = struct.unpack_from(">H", data, offset)[0]
    offset += 2 + 2 * interface_count
    field_count = struct.unpack_from(">H", data, offset)[0]
    offset += 2

    def skip_member(at: int) -> tuple[int, str, str, list[tuple[str, int, int, int]]]:
        _, name_index, desc_index, attributes = struct.unpack_from(">HHHH", data, at)
        at += 8
        rows = []
        for _ in range(attributes):
            attr_index, length = struct.unpack_from(">HI", data, at)
            body = at + 6
            rows.append((utf8[attr_index], at, body, length))
            at = body + length
        return at, utf8[name_index], utf8[desc_index], rows

    for _ in range(field_count):
        offset, _, _, _ = skip_member(offset)
    method_count = struct.unpack_from(">H", data, offset)[0]
    offset += 2
    if method_count != 1:
        raise ValueError("expected the one original test method")
    _, method_name, descriptor, attributes = skip_member(offset)
    if (method_name, descriptor) != ("test", "([I[I)V"):
        raise ValueError("the frozen target method identity changed")
    code_attrs = [row for row in attributes if row[0] == "Code"]
    if len(code_attrs) != 1:
        raise ValueError("expected one Code attribute")
    _, attr_start, content_start, attr_length = code_attrs[0]
    code_length = struct.unpack_from(">I", data, content_start + 4)[0]
    code_start = content_start + 8
    old_code = bytes(data[code_start : code_start + code_length])
    if old_code != ORIGINAL_CODE:
        raise ValueError("the frozen original Code bytes changed")
    tail = bytes(data[code_start + code_length : content_start + attr_length])
    delta = len(replacement) - len(old_code)
    return (
        bytes(data[: attr_start + 2])
        + struct.pack(">I", attr_length + delta)
        + bytes(data[content_start : content_start + 4])
        + struct.pack(">I", len(replacement))
        + replacement
        + tail
        + bytes(data[content_start + attr_length :])
    )


if __name__ == "__main__":
    source = SOURCE.read_bytes()
    if source[4:8] != bytes.fromhex("0003002d"):
        raise SystemExit("expected class-file version 45.3")
    output_dir = ROOT / "controls"
    output_dir.mkdir(parents=True, exist_ok=True)
    for name, code in OUTPUTS.items():
        (output_dir / name).write_bytes(replace_code(source, code))
