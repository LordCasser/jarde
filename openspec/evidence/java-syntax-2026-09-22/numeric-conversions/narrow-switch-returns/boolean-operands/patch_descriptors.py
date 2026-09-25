"""Patch BooleanOperands.run from (Z)Z to the verifier-compatible B/C/S returns."""

from pathlib import Path
import hashlib
import json
import struct
import sys


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
            offset += 2 + u2(data, offset)
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
    length = u2(entry, 1)
    return entry[3 : 3 + length].decode("utf-8")


def skip_attributes(data, offset):
    count = u2(data, offset)
    offset += 2
    for _ in range(count):
        offset += 6 + struct.unpack_from(">I", data, offset + 2)[0]
    return offset


def method_span(data, entries, cp_end):
    offset = cp_end + 6
    interfaces = u2(data, offset)
    offset += 2 + interfaces * 2
    fields = u2(data, offset)
    offset += 2
    for _ in range(fields):
        offset = skip_attributes(data, offset + 6)
    methods = u2(data, offset)
    offset += 2
    for _ in range(methods):
        start = offset
        name = utf8(entries, u2(data, offset + 2))
        descriptor_index = u2(data, offset + 4)
        end = skip_attributes(data, offset + 6)
        if name == "run":
            return {
                "start": start,
                "descriptor_offset": offset + 4,
                "descriptor": utf8(entries, descriptor_index),
                "end": end,
            }
        offset = end
    raise ValueError("missing run")


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


def patch(input_path, output_path, record_path, return_kind):
    original = Path(input_path).read_bytes()
    original_code = code_attributes(original)
    entries, cp_end = cp_info(original)
    span = method_span(original, entries, cp_end)
    if span["descriptor"] != "(Z)Z":
        raise ValueError(f"run has {span['descriptor']}, expected (Z)Z")
    target = f"(Z){return_kind}"
    existing = {
        utf8(entries, index)
        for index in range(1, len(entries))
        if entries[index] and entries[index][0] == 1
    }
    if target in existing:
        descriptor_index = next(
            index for index in range(1, len(entries))
            if entries[index] and entries[index][0] == 1 and utf8(entries, index) == target
        )
        additions = b""
    else:
        descriptor_index = len(entries)
        encoded = target.encode("utf-8")
        additions = b"\x01" + struct.pack(">H", len(encoded)) + encoded
    body = bytearray(original[cp_end:])
    struct.pack_into(">H", body, span["descriptor_offset"] - cp_end, descriptor_index)
    patched = (
        original[:8]
        + struct.pack(">H", u2(original, 8) + (1 if additions else 0))
        + original[10:cp_end]
        + additions
        + bytes(body)
    )
    patched_code = code_attributes(patched)
    if original_code != patched_code:
        raise ValueError("descriptor patch changed Code attributes")
    Path(output_path).write_bytes(patched)
    Path(record_path).write_text(
        json.dumps(
            {
                "from": "(Z)Z",
                "to": target,
                "descriptor_offset": span["descriptor_offset"],
                "source_bytes": len(original),
                "patched_bytes": len(patched),
                "source_sha256": hashlib.sha256(original).hexdigest(),
                "patched_sha256": hashlib.sha256(patched).hexdigest(),
                "code_attributes": len(original_code),
                "code_sha256_before": hashlib.sha256(b"".join(original_code)).hexdigest(),
                "code_sha256_after": hashlib.sha256(b"".join(patched_code)).hexdigest(),
            },
            indent=2,
        )
        + "\n"
    )


if __name__ == "__main__":
    if len(sys.argv) != 5:
        raise SystemExit("usage: patch_descriptors.py INPUT.class OUTPUT.class PATCH.json B|C|S")
    patch(*sys.argv[1:])
