#!/usr/bin/env python3
"""Patch one pinned branch operand in the frozen Java 8 class fixture."""
from __future__ import annotations

import hashlib
import json
import struct
import sys
from pathlib import Path

EXPECTED_SHA256 = "2fc584d5ba9d35940516fcf9001479a72851ebb0216c22a62f3ec5500dbc35aa"
METHOD = "test"
DESCRIPTOR = "(ZZZZLjava/lang/StringBuilder;)Ljava/lang/String;"
BRANCH_BCI = 59
OLD_OPCODE = 0x99  # ifeq
OLD_DELTA = 15
NEW_TARGET = 40


def u2(data: bytes, offset: int) -> int:
    return struct.unpack_from(">H", data, offset)[0]


def u4(data: bytes, offset: int) -> int:
    return struct.unpack_from(">I", data, offset)[0]


def cp_utf8(data: bytes, cp_offset: int) -> tuple[dict[int, str], int]:
    count = u2(data, cp_offset)
    pos = cp_offset + 2
    values: dict[int, str] = {}
    index = 1
    while index < count:
        tag = data[pos]
        pos += 1
        if tag == 1:
            length = u2(data, pos)
            pos += 2
            values[index] = data[pos:pos + length].decode("utf-8", "strict")
            pos += length
        elif tag in (3, 4):
            pos += 4
        elif tag in (5, 6):
            pos += 8
            index += 1
        elif tag in (7, 8, 16, 19, 20):
            pos += 2
        elif tag in (9, 10, 11, 12, 17, 18):
            pos += 4
        elif tag == 15:
            pos += 3
        else:
            raise ValueError(f"unsupported constant-pool tag {tag} at index {index}")
        index += 1
    return values, pos


def skip_member(data: bytes, pos: int) -> int:
    attr_count = u2(data, pos + 6)
    pos += 8
    for _ in range(attr_count):
        length = u4(data, pos + 2)
        pos += 6 + length
    return pos


def code_attribute(data: bytes) -> tuple[int, int, int, list[tuple[str, int, int]]]:
    if data[:4] != b"\xca\xfe\xba\xbe":
        raise ValueError("not a class file")
    cp, cp_end = cp_utf8(data, 8)
    pos = cp_end + 6  # access_flags, this_class, super_class
    interfaces = u2(data, pos)
    pos += 2 + 2 * interfaces
    fields = u2(data, pos)
    pos += 2
    for _ in range(fields):
        pos = skip_member(data, pos)
    methods = u2(data, pos)
    pos += 2
    for _ in range(methods):
        name = cp[u2(data, pos + 2)]
        descriptor = cp[u2(data, pos + 4)]
        attrs = u2(data, pos + 6)
        attr_pos = pos + 8
        for _ in range(attrs):
            attr_name = cp[u2(data, attr_pos)]
            attr_length = u4(data, attr_pos + 2)
            body = attr_pos + 6
            if name == METHOD and descriptor == DESCRIPTOR and attr_name == "Code":
                code_length = u4(data, body + 4)
                code_start = body + 8
                ex_count_pos = code_start + code_length
                ex_count = u2(data, ex_count_pos)
                nested_count_pos = ex_count_pos + 2 + ex_count * 8
                nested_count = u2(data, nested_count_pos)
                nested_pos = nested_count_pos + 2
                nested: list[tuple[str, int, int]] = []
                for _ in range(nested_count):
                    nested_name = cp[u2(data, nested_pos)]
                    nested_length = u4(data, nested_pos + 2)
                    nested.append((nested_name, nested_pos + 6, nested_length))
                    nested_pos += 6 + nested_length
                if nested_pos != body + attr_length:
                    raise ValueError("Code attribute layout did not close exactly")
                return code_start, code_length, body, nested
            attr_pos += 6 + attr_length
        pos = attr_pos
    raise ValueError(f"pinned method {METHOD}{DESCRIPTOR} Code attribute not found")


def instruction_starts(code: bytes) -> set[int]:
    starts: set[int] = set()
    pos = 0
    while pos < len(code):
        starts.add(pos)
        op = code[pos]
        if op in (0xaa, 0xab):
            pad = (4 - ((pos + 1) & 3)) & 3
            base = pos + 1 + pad
            if op == 0xaa:
                low, high = struct.unpack_from(">ii", code, base + 4)
                size = base + 12 + (high - low + 1) * 4 - pos
            else:
                pairs = struct.unpack_from(">i", code, base + 4)[0]
                size = base + 8 + pairs * 8 - pos
        elif op == 0xc4:
            size = 6 if code[pos + 1] == 0x84 else 4
        elif op in (0x10, 0x12, 0x15, 0x16, 0x17, 0x18, 0x19,
                    0x36, 0x37, 0x38, 0x39, 0x3a, 0xa9, 0xbc):
            size = 2
        elif op in (0x11, 0x13, 0x14, 0x84, 0x99, 0x9a, 0x9b, 0x9c,
                    0x9d, 0x9e, 0x9f, 0xa0, 0xa1, 0xa2, 0xa3, 0xa4,
                    0xa5, 0xa6, 0xa7, 0xa8, 0xc6, 0xc7, 0xc8, 0xc9,
                    0xb2, 0xb3, 0xb4, 0xb5, 0xb6, 0xb7, 0xb8, 0xbb,
                    0xbd, 0xc0, 0xc1, 0xc5):
            size = 3 if op not in (0xc5,) else 4
        elif op in (0xb9, 0xba):
            size = 5
        else:
            size = 1
        if size <= 0 or pos + size > len(code):
            raise ValueError(f"invalid instruction width at BCI {pos}")
        pos += size
    if pos != len(code):
        raise ValueError("instruction scan does not end at Code length")
    return starts


def stack_map_offsets(data: bytes) -> set[int]:
    count = u2(data, 0)
    pos = 2
    previous = -1
    offsets: set[int] = set()

    def verification_type(p: int) -> int:
        tag = data[p]
        if tag <= 6:
            return p + 1
        if tag in (7, 8):
            return p + 3
        raise ValueError(f"invalid verification_type_info tag {tag}")

    for _ in range(count):
        frame = data[pos]
        pos += 1
        if frame <= 63:
            delta = frame
        elif frame <= 127:
            delta = frame - 64
            pos = verification_type(pos)
        elif frame == 247:
            delta = u2(data, pos)
            pos += 2
            pos = verification_type(pos)
        elif 248 <= frame <= 251:
            delta = u2(data, pos)
            pos += 2
        elif 252 <= frame <= 254:
            delta = u2(data, pos)
            pos += 2
            for _ in range(frame - 251):
                pos = verification_type(pos)
        elif frame == 255:
            delta = u2(data, pos)
            locals_count = u2(data, pos + 2)
            pos += 4
            for _ in range(locals_count):
                pos = verification_type(pos)
            stack_count = u2(data, pos)
            pos += 2
            for _ in range(stack_count):
                pos = verification_type(pos)
        else:
            raise ValueError(f"reserved StackMapTable frame_type {frame}")
        current = previous + delta + 1
        offsets.add(current)
        previous = current
    if pos != len(data):
        raise ValueError("StackMapTable did not close exactly")
    return offsets


def main() -> None:
    if len(sys.argv) != 3:
        raise SystemExit("usage: patch_cycle.py frozen.class output.class")
    source = Path(sys.argv[1]).read_bytes()
    source_hash = hashlib.sha256(source).hexdigest()
    if source_hash != EXPECTED_SHA256:
        raise SystemExit(f"source SHA-256 mismatch: {source_hash}")
    code_start, code_length, _body, nested = code_attribute(source)
    code = source[code_start:code_start + code_length]
    starts = instruction_starts(code)
    if BRANCH_BCI not in starts or code[BRANCH_BCI] != OLD_OPCODE:
        raise SystemExit("pinned BCI 59 is not the expected ifeq opcode")
    old_delta = struct.unpack_from(">h", code, BRANCH_BCI + 1)[0]
    if old_delta != OLD_DELTA or BRANCH_BCI + old_delta != 74:
        raise SystemExit(f"pinned old target mismatch: delta={old_delta}")
    if NEW_TARGET not in starts:
        raise SystemExit("target BCI 40 is not an instruction boundary")
    stack_maps = [raw for name, offset, length in nested
                  if name == "StackMapTable"
                  for raw in [source[offset:offset + length]]]
    if len(stack_maps) != 1 or NEW_TARGET not in stack_map_offsets(stack_maps[0]):
        raise SystemExit("target BCI 40 lacks its existing StackMapTable frame")
    new_delta = NEW_TARGET - BRANCH_BCI
    if not -(1 << 15) <= new_delta < (1 << 15):
        raise SystemExit("new branch displacement does not fit signed short")
    patched = bytearray(source)
    operand_offset = code_start + BRANCH_BCI + 1
    patched[operand_offset:operand_offset + 2] = struct.pack(">h", new_delta)
    output = Path(sys.argv[2])
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_bytes(patched)
    changes = [index for index, (before, after) in enumerate(zip(source, patched))
               if before != after]
    if changes != [operand_offset, operand_offset + 1]:
        output.unlink(missing_ok=True)
        raise SystemExit(f"unexpected byte diff offsets: {changes}")
    manifest = {
        "source": str(Path(sys.argv[1]).resolve()),
        "source_sha256": source_hash,
        "source_bytes": len(source),
        "patched_sha256": hashlib.sha256(patched).hexdigest(),
        "patched_bytes": len(patched),
        "method": METHOD,
        "descriptor": DESCRIPTOR,
        "branch_bci": BRANCH_BCI,
        "opcode": "ifeq",
        "old_signed_delta": OLD_DELTA,
        "old_target_bci": BRANCH_BCI + OLD_DELTA,
        "new_signed_delta": new_delta,
        "new_target_bci": NEW_TARGET,
        "changed_class_offsets": changes,
        "target_has_existing_stackmap_frame": True,
        "purpose": "reader-produced normal-flow cycle refusal; never invoke this method",
    }
    output.with_suffix(output.suffix + ".json").write_text(
        json.dumps(manifest, indent=2) + "\n", encoding="utf-8"
    )


if __name__ == "__main__":
    main()
