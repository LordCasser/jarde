from pathlib import Path
import hashlib
import json
import struct
import sys


PATCHES = {
    "directByte": "(Ljava/lang/String;I)B",
    "directChar": "(Ljava/lang/Number;I)C",
    "directShort": "(ZI)S",
    "byteLocal": "(B)B",
    "charLocal": "(C)C",
    "shortLocal": "(S)S",
    "postByte": "(I)B",
    "preChar": "(J)C",
    "postShort": "(F)S",
    "syncByte": "(Ljava/lang/Object;I)B",
    "syncChar": "(Ljava/lang/String;I)C",
    "syncShort": "(Ljava/lang/Object;JI)S",
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


def skip_attributes(data, offset, entries):
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
        offset = skip_attributes(data, offset + 6, entries)
    methods = u2(data, offset)
    offset += 2
    spans = {}
    for _ in range(methods):
        start = offset
        name_index = u2(data, offset + 2)
        descriptor_index = u2(data, offset + 4)
        name = utf8(entries, name_index)
        attribute_end = skip_attributes(data, offset + 6, entries)
        spans[name] = {
            "start": start,
            "descriptor_offset": offset + 4,
            "descriptor": utf8(entries, descriptor_index),
            "end": attribute_end,
        }
        offset = attribute_end
    return spans


def code_attributes(data):
    entries, cp_end = cp_info(data)
    offset = cp_end + 6
    interfaces = u2(data, offset)
    offset += 2 + interfaces * 2
    fields = u2(data, offset)
    offset += 2
    for _ in range(fields):
        offset = skip_attributes(data, offset + 6, entries)
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
    descriptor_by_name = {}
    patch_records = []
    for name, target_descriptor in PATCHES.items():
        if name not in spans:
            raise ValueError(f"missing method {name}")
        source_descriptor = spans[name]["descriptor"]
        if not source_descriptor.endswith(")I"):
            raise ValueError(f"{name} has unexpected descriptor {source_descriptor}")
        descriptor_by_name[name] = target_descriptor
        patch_records.append(
            {
                "method": name,
                "from": source_descriptor,
                "to": target_descriptor,
                "code_span": [spans[name]["start"], spans[name]["end"]],
            }
        )

    existing = {utf8(entries, index) for index in range(1, len(entries)) if entries[index] and entries[index][0] == 1}
    additions = []
    index_by_descriptor = {}
    for descriptor in descriptor_by_name.values():
        if descriptor not in index_by_descriptor:
            if descriptor in existing:
                index_by_descriptor[descriptor] = next(
                    index for index in range(1, len(entries)) if entries[index] and entries[index][0] == 1 and utf8(entries, index) == descriptor
                )
            else:
                encoded = descriptor.encode("utf-8")
                index_by_descriptor[descriptor] = len(entries) + len(additions)
                additions.append(b"\x01" + struct.pack(">H", len(encoded)) + encoded)

    body = bytearray(original[cp_end:])
    for record in patch_records:
        relative = record["code_span"][0] - cp_end + 4
        descriptor = index_by_descriptor[record["to"]]
        struct.pack_into(">H", body, relative, descriptor)
        record["descriptor_index"] = descriptor

    cp_count = u2(original, 8) + len(additions)
    prefix = bytearray(original[:8]) + struct.pack(">H", cp_count)
    old_pool = original[10:cp_end]
    patched = bytes(prefix) + old_pool + b"".join(additions) + bytes(body)
    patched_code = code_attributes(patched)
    if original_code != patched_code:
        raise ValueError("descriptor patch changed Code attributes")
    Path(output_path).write_bytes(patched)
    record = {
        "code_attributes": len(original_code),
        "code_sha256_before": hashlib.sha256(b"".join(original_code)).hexdigest(),
        "code_sha256_after": hashlib.sha256(b"".join(patched_code)).hexdigest(),
        "patches": patch_records,
    }
    Path(record_path).write_text(json.dumps(record, indent=2) + "\n")


if __name__ == "__main__":
    if len(sys.argv) != 4:
        raise SystemExit("usage: patch_descriptors.py INPUT.class OUTPUT.class PATCHES.json")
    patch(*sys.argv[1:])
