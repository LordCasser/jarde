#!/usr/bin/env python3
"""Apply narrowly scoped, verifier-checkable array descriptor/opcode patches."""

from __future__ import annotations

import argparse
import hashlib
import json
import struct
from pathlib import Path


class ClassFile:
    def __init__(self, data: bytes):
        self.data = bytearray(data)
        self.cp, self.cp_end, self.cp_entries = self._constant_pool()
        if self.data[:4] != b"\xca\xfe\xba\xbe":
            raise ValueError("input is not a class file")
        major = struct.unpack_from(">H", self.data, 6)[0]
        if major != 52:
            raise ValueError(f"expected Java 8 class version 52, got {major}")

    def u2(self, offset: int) -> int:
        return struct.unpack_from(">H", self.data, offset)[0]

    def u4(self, offset: int) -> int:
        return struct.unpack_from(">I", self.data, offset)[0]

    def _constant_pool(self):
        count = self.u2(8)
        entries = [None] * count
        spans = [None] * count
        offset = 10
        index = 1
        while index < count:
            start = offset
            tag = self.data[offset]
            offset += 1
            if tag == 1:
                length = self.u2(offset)
                offset += 2
                entries[index] = bytes(self.data[offset:offset + length])
                spans[index] = (offset, length)
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
                raise ValueError(f"unknown constant-pool tag {tag} at {start}")
            index += 1
        return entries, offset, spans

    def replace_utf8(self, old: bytes, new: bytes) -> int:
        if len(old) != len(new):
            raise ValueError(f"descriptor lengths differ: {old!r}, {new!r}")
        matches = [(index, span) for index, (value, span) in enumerate(zip(self.cp, self.cp_entries)) if value == old]
        if not matches:
            raise ValueError(f"constant-pool UTF8 not found: {old!r}")
        for index, (start, length) in matches:
            if length != len(old):
                raise ValueError("unexpected UTF8 length")
            self.data[start:start + length] = new
            self.cp[index] = new
        return len(matches)

    def methods(self):
        offset = self.cp_end + 6
        interfaces = self.u2(offset)
        offset += 2 + 2 * interfaces
        fields = self.u2(offset)
        offset += 2
        for _ in range(fields):
            offset = self._skip_member(offset)
        count = self.u2(offset)
        offset += 2
        result = []
        for _ in range(count):
            access = self.u2(offset)
            name_index = self.u2(offset + 2)
            desc_index = self.u2(offset + 4)
            attribute_count = self.u2(offset + 6)
            offset += 8
            attrs = []
            for _ in range(attribute_count):
                name = self.cp[self.u2(offset)]
                length = self.u4(offset + 2)
                data = offset + 6
                attrs.append((name, data, length))
                offset = data + length
            result.append((self.cp[name_index], self.cp[desc_index], access, attrs))
        return result

    def _skip_member(self, offset):
        attribute_count = self.u2(offset + 6)
        offset += 8
        for _ in range(attribute_count):
            offset += 6 + self.u4(offset + 2)
        return offset


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def patch_fixture(name: str, source: Path, destination: Path, manifest_path: Path) -> None:
    raw = source.read_bytes()
    class_file = ClassFile(raw)
    if name == "RawBool":
        descriptor_edits = [(b"([III)I", b"([ZII)Z")]
    elif name == "Order":
        descriptor_edits = [
            (b"([I)[I", b"([Z)[Z"),
            (b"([II)I", b"([ZI)Z"),
        ]
    else:
        raise ValueError(f"unsupported fixture {name!r}")

    descriptor_counts = []
    for old, new in descriptor_edits:
        descriptor_counts.append({
            "from": old.decode("ascii"),
            "to": new.decode("ascii"),
            "constant_pool_entries": class_file.replace_utf8(old, new),
        })

    methods = class_file.methods()
    target = [method for method in methods if method[0] == b"put"]
    if len(target) != 1:
        raise ValueError(f"expected one put method in {name}")
    method_name, descriptor, access, attrs = target[0]
    code_attrs = [item for item in attrs if item[0] == b"Code"]
    if len(code_attrs) != 1:
        raise ValueError(f"expected one Code attribute for {name}.put")
    _, data, _ = code_attrs[0]
    code_length = class_file.u4(data + 4)
    code_start = data + 8
    code_end = code_start + code_length
    code = bytes(class_file.data[code_start:code_end])
    expected = {
        "RawBool": {0x4F: 1, 0x2E: 1},
        "Order": {0x4F: 1, 0x2E: 1},
    }[name]
    actual_edits = []
    for old_opcode, new_opcode in ((0x4F, 0x54), (0x2E, 0x33)):
        positions = [i for i, opcode in enumerate(code) if opcode == old_opcode]
        if len(positions) != expected[old_opcode]:
            raise ValueError(
                f"{name}.put expected {expected[old_opcode]} opcode 0x{old_opcode:02x}; "
                f"found positions {positions} in code {code.hex()}"
            )
        for position in positions:
            class_file.data[code_start + position] = new_opcode
        actual_edits.append({
            "offset_in_code": positions,
            "from": f"0x{old_opcode:02x}",
            "to": f"0x{new_opcode:02x}",
        })

    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_bytes(class_file.data)
    manifest = {
        "class": name,
        "classfile_major_version": 52,
        "original": {"path": str(source), "sha256": digest(source), "bytes": len(raw)},
        "patched": {
            "path": str(destination),
            "sha256": digest(destination),
            "bytes": len(class_file.data),
        },
        "descriptor_edits": descriptor_counts,
        "method": {
            "name": method_name.decode("ascii"),
            "patched_descriptor": descriptor.decode("ascii"),
            "access_flags": f"0x{access:04x}",
            "code_length": code_length,
            "original_code_hex": code.hex(),
            "array_opcode_edits": actual_edits,
        },
        "scope": "Only the stated method descriptor(s) and the sole iastore/iaload pair in put are changed.",
    }
    manifest_path.parent.mkdir(parents=True, exist_ok=True)
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("fixture", choices=("RawBool", "Order"))
    parser.add_argument("--source", required=True, type=Path)
    parser.add_argument("--destination", required=True, type=Path)
    parser.add_argument("--manifest", required=True, type=Path)
    args = parser.parse_args()
    patch_fixture(args.fixture, args.source, args.destination, args.manifest)


if __name__ == "__main__":
    main()
