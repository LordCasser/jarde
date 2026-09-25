#!/usr/bin/env python3
"""Remove declaration annotation attributes from the three scalar members."""
import hashlib
import struct
import sys
from pathlib import Path


def u2(data, offset):
    return struct.unpack_from(">H", data, offset)[0]


def u4(data, offset):
    return struct.unpack_from(">I", data, offset)[0]


def parse_constant_pool(data):
    count = u2(data, 8)
    offset = 10
    utf8 = {}
    index = 1
    while index < count:
        tag = data[offset]
        offset += 1
        if tag == 1:
            size = u2(data, offset)
            offset += 2
            utf8[index] = data[offset:offset + size].decode("utf-8")
            offset += size
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
    return offset, utf8


def parse_attributes(data, offset, utf8):
    count = u2(data, offset)
    start = offset
    offset += 2
    entries = []
    for _ in range(count):
        name_index = u2(data, offset)
        size = u4(data, offset + 2)
        end = offset + 6 + size
        entries.append((utf8[name_index], data[offset:end], data[offset + 6:end]))
        offset = end
    return data[start:offset], offset, entries


def members(data, offset, utf8, kind, remove_targets=None, code_hashes=None):
    count = u2(data, offset)
    out = bytearray(data[offset:offset + 2])
    offset += 2
    removed = []
    for _ in range(count):
        header = data[offset:offset + 6]
        name = utf8[u2(data, offset + 2)]
        descriptor = utf8[u2(data, offset + 4)]
        offset += 6
        _, offset, attrs = parse_attributes(data, offset, utf8)
        target = (kind, name, descriptor)
        remove = remove_targets.get(target, set()) if remove_targets else set()
        kept = []
        for attr_name, raw, info in attrs:
            if attr_name in remove:
                removed.append(f"{kind} {name}{descriptor}: {attr_name}")
                continue
            kept.append(raw)
            if attr_name == "Code" and code_hashes is not None:
                code_hashes[f"{kind} {name}{descriptor}"] = hashlib.sha256(info).hexdigest()
        out += header + struct.pack(">H", len(kept)) + b"".join(kept)
    return bytes(out), offset, removed


def main(source, destination):
    data = Path(source).read_bytes()
    if data[:4] != b"\xca\xfe\xba\xbe":
        raise ValueError("input is not a class file")
    offset, utf8 = parse_constant_pool(data)
    fixed = data[:offset]
    # access_flags, this_class, super_class, interfaces_count and interfaces
    interface_count = u2(data, offset + 6)
    after_interfaces = offset + 8 + 2 * interface_count
    prefix = data[offset:after_interfaces]
    field_remove = {("field", "field", "I"): {"RuntimeVisibleAnnotations"}}
    fields, method_start, removed_fields = members(data, after_interfaces, utf8, "field", field_remove)
    method_remove = {
        ("method", "answer", "()I"): {"RuntimeVisibleAnnotations"},
        ("method", "echo", "(I)I"): {"RuntimeVisibleParameterAnnotations"},
    }
    code_before = {}
    methods, cursor, removed_methods = members(data, method_start, utf8, "method",
                                                method_remove, code_before)
    class_attrs_raw, end, _ = parse_attributes(data, cursor, utf8)
    if end != len(data):
        raise ValueError("unexpected trailing bytes")
    # Class-level attributes (including the annotation declaration's own attributes) are unchanged.
    output = fixed + prefix + fields + methods + class_attrs_raw
    Path(destination).parent.mkdir(parents=True, exist_ok=True)
    Path(destination).write_bytes(output)
    patched_cp_end, patched_utf8 = parse_constant_pool(output)
    patched_interfaces = u2(output, patched_cp_end + 6)
    patched_after_interfaces = patched_cp_end + 8 + 2 * patched_interfaces
    _, patched_method_start, _ = members(output, patched_after_interfaces, patched_utf8, "field")
    code_after = {}
    _, _, _ = members(output, patched_method_start, patched_utf8, "method", code_hashes=code_after)
    # Persist an auditable report beside the generated class.
    lines = ["removed attributes:"] + ["  " + x for x in removed_fields + removed_methods]
    lines += ["Code attribute-info SHA-256 (baseline / patched):"]
    for member, digest in sorted(code_before.items()):
        after = code_after.get(member)
        lines.append(f"  {member}\t{digest}\t{after}\t{'IDENTICAL' if digest == after else 'DIFFERENT'}")
    Path(destination).with_suffix(".patch-report.txt").write_text("\n".join(lines) + "\n")


if __name__ == "__main__":
    if len(sys.argv) != 3:
        raise SystemExit("usage: patch_scalar_class.py INPUT.class OUTPUT.class")
    main(sys.argv[1], sys.argv[2])
