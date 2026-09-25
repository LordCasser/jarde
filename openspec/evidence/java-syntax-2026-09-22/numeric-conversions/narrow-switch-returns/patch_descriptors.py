"""Patch only selected method descriptors, preserving every Code attribute byte."""

from pathlib import Path
import hashlib
import json
import struct
import sys


PATCHES = {
    "runByte": (("(I)I", "(I)B")),
    "runChar": (("(I)I", "(I)C")),
    "runShort": (("(I)I", "(I)S")),
    "boolByte": (("(Z)I", "(Z)B")),
    "boolChar": (("(Z)I", "(Z)C")),
    "boolShort": (("(Z)I", "(Z)S")),
}


def u2(data, offset):
    return struct.unpack_from(">H", data, offset)[0]


def cp_info(data):
    count = u2(data, 8)
    entries = [None] * count
    offset = 10
    index = 1
    while index < count:
        start = offset
        tag = data[offset]
        offset += 1
        if tag == 1:
            length = u2(data, offset)
            offset += 2 + length
        elif tag in {3, 4, 9, 10, 11, 12, 17, 18}:
            offset += 4
        elif tag in {7, 8, 16, 19, 20}:
            offset += 2
        elif tag == 15:
            offset += 3
        elif tag in {5, 6}:
            offset += 8
        else:
            raise ValueError(f"unsupported constant-pool tag {tag}")
        entries[index] = data[start:offset]
        index += 1
        if tag in {5, 6}:
            index += 1
    return entries, offset


def utf8(entries, index):
    entry = entries[index]
    if entry[0] != 1:
        raise ValueError("constant-pool entry is not Utf8")
    length = struct.unpack_from(">H", entry, 1)[0]
    return entry[3 : 3 + length].decode("utf-8")


def skip_attributes(data, offset):
    count = u2(data, offset)
    offset += 2
    for _ in range(count):
        length = struct.unpack_from(">I", data, offset + 2)[0]
        offset += 6 + length
    return offset


def method_spans(data, entries, cp_end):
    offset = cp_end + 6
    interfaces = u2(data, offset)
    offset += 2 + interfaces * 2
    fields = u2(data, offset)
    offset += 2
    for _ in range(fields):
        offset = skip_attributes(data, offset + 6)
    methods = u2(data, offset)
    offset += 2
    spans = {}
    for _ in range(methods):
        start = offset
        name_index = u2(data, offset + 2)
        descriptor_index = u2(data, offset + 4)
        name = utf8(entries, name_index)
        end = skip_attributes(data, offset + 6)
        spans[name] = {
            "start": start,
            "descriptor_offset": offset + 4,
            "descriptor": utf8(entries, descriptor_index),
            "end": end,
        }
        offset = end
    return spans


def code_attributes(data):
    entries, cp_end = cp_info(data)
    offset = cp_end + 6
    interfaces = u2(data, offset)
    offset += 2 + interfaces * 2
    fields = u2(data, offset)
    offset += 2
    for _ in range(fields):
        offset = skip_attributes(data, offset + 6)
    methods = u2(data, offset)
    offset += 2
    result = []
    for _ in range(methods):
        attributes = u2(data, offset + 6)
        attribute_offset = offset + 8
        for _ in range(attributes):
            name_index = u2(data, attribute_offset)
            length = struct.unpack_from(">I", data, attribute_offset + 2)[0]
            end = attribute_offset + 6 + length
            if utf8(entries, name_index) == "Code":
                result.append(data[attribute_offset:end])
            attribute_offset = end
        offset = attribute_offset
    return result


def patch(input_path, output_path, record_path):
    original = Path(input_path).read_bytes()
    original_code = code_attributes(original)
    entries, cp_end = cp_info(original)
    spans = method_spans(original, entries, cp_end)
    patch_records = []
    descriptors = {}
    for name, (source_descriptor, target_descriptor) in PATCHES.items():
        if name not in spans:
            raise ValueError(f"missing method {name}")
        actual = spans[name]["descriptor"]
        if actual != source_descriptor:
            raise ValueError(f"{name} has {actual}, expected {source_descriptor}")
        descriptors[name] = target_descriptor
        patch_records.append(
            {
                "method": name,
                "from": actual,
                "to": target_descriptor,
                "method_span": [spans[name]["start"], spans[name]["end"]],
                "descriptor_offset": spans[name]["descriptor_offset"],
            }
        )

    existing = {
        utf8(entries, index)
        for index in range(1, len(entries))
        if entries[index] and entries[index][0] == 1
    }
    additions = []
    index_by_descriptor = {}
    for descriptor in descriptors.values():
        if descriptor in index_by_descriptor:
            continue
        if descriptor in existing:
            index_by_descriptor[descriptor] = next(
                index
                for index in range(1, len(entries))
                if entries[index]
                and entries[index][0] == 1
                and utf8(entries, index) == descriptor
            )
        else:
            encoded = descriptor.encode("utf-8")
            index_by_descriptor[descriptor] = len(entries) + len(additions)
            additions.append(b"\x01" + struct.pack(">H", len(encoded)) + encoded)

    body = bytearray(original[cp_end:])
    for record in patch_records:
        relative = record["descriptor_offset"] - cp_end
        descriptor_index = index_by_descriptor[record["to"]]
        struct.pack_into(">H", body, relative, descriptor_index)
        record["descriptor_index"] = descriptor_index

    cp_count = u2(original, 8) + len(additions)
    prefix = bytearray(original[:8]) + struct.pack(">H", cp_count)
    old_pool = original[10:cp_end]
    patched = bytes(prefix) + old_pool + b"".join(additions) + bytes(body)
    patched_code = code_attributes(patched)
    if original_code != patched_code:
        raise ValueError("descriptor patch changed Code attributes")
    Path(output_path).write_bytes(patched)
    Path(record_path).write_text(
        json.dumps(
            {
                "input_bytes": len(original),
                "output_bytes": len(patched),
                "code_attributes": len(original_code),
                "code_sha256_before": hashlib.sha256(b"".join(original_code)).hexdigest(),
                "code_sha256_after": hashlib.sha256(b"".join(patched_code)).hexdigest(),
                "patches": patch_records,
            },
            indent=2,
        )
        + "\n"
    )


if __name__ == "__main__":
    if len(sys.argv) != 4:
        raise SystemExit("usage: patch_descriptors.py INPUT.class OUTPUT.class PATCHES.json")
    patch(*sys.argv[1:])
