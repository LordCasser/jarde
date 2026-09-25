#!/usr/bin/env python3
"""Patch only selected field_info and Fieldref NameAndType descriptors."""
from __future__ import annotations

import hashlib
import json
import struct
import sys
from pathlib import Path

TARGETS = {
    "byteField": "B",
    "charField": "C",
    "shortField": "S",
    "staticByteField": "B",
    "staticCharField": "C",
    "staticShortField": "S",
}


def u1(data, off):
    return data[off], off + 1


def u2(data, off):
    return struct.unpack_from(">H", data, off)[0], off + 2


def u4(data, off):
    return struct.unpack_from(">I", data, off)[0], off + 4


def put_u2(data, off, value):
    struct.pack_into(">H", data, off, value)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def parse_cp(data):
    count = struct.unpack_from(">H", data, 8)[0]
    entries = [None] * count
    off = 10
    index = 1
    while index < count:
        tag, off = u1(data, off)
        start = off - 1
        if tag == 1:
            length, off = u2(data, off)
            raw = bytes(data[off:off + length])
            entries[index] = (tag, raw, start)
            off += length
        elif tag in (3, 4):
            entries[index] = (tag, bytes(data[off:off + 4]), start)
            off += 4
        elif tag in (5, 6):
            entries[index] = (tag, bytes(data[off:off + 8]), start)
            off += 8
            index += 1
        elif tag in (7, 8, 16, 19, 20):
            entries[index] = (tag, bytes(data[off:off + 2]), start)
            off += 2
        elif tag in (9, 10, 11, 12, 17, 18):
            entries[index] = (tag, bytes(data[off:off + 4]), start)
            off += 4
        elif tag == 15:
            entries[index] = (tag, bytes(data[off:off + 3]), start)
            off += 3
        else:
            raise ValueError(f"unknown constant-pool tag {tag} at {index}")
        index += 1
    return entries, off


def utf8(entries, index):
    item = entries[index]
    if item is None or item[0] != 1:
        raise ValueError(f"constant {index} is not Utf8")
    return item[1].decode("utf-8")


def add_utf8(data, entries, cp_end, value):
    encoded = value.encode("utf-8")
    if len(encoded) > 0xFFFF:
        raise ValueError("Utf8 constant too long")
    old_count = struct.unpack_from(">H", data, 8)[0]
    if old_count >= 0xFFFF:
        raise ValueError("constant pool full")
    index = len(entries)
    addition = bytes([1]) + struct.pack(">H", len(encoded)) + encoded
    data = data[:8] + struct.pack(">H", old_count + 1) + data[10:cp_end] + addition + data[cp_end:]
    entries.append((1, encoded, None))
    return data, entries, cp_end + len(addition), index


def skip_attributes(data, off, count):
    attrs = []
    for _ in range(count):
        name_index, off = u2(data, off)
        length, off = u4(data, off)
        attrs.append((name_index, off, length))
        off += length
    return attrs, off


def class_layout(data, entries):
    _, cp_end = parse_cp(data)
    off = cp_end
    access, off = u2(data, off)
    this_class, off = u2(data, off)
    super_class, off = u2(data, off)
    interface_count, off = u2(data, off)
    off += 2 * interface_count
    field_count, off = u2(data, off)
    fields = []
    for _ in range(field_count):
        info = off
        field_access, off = u2(data, off)
        name_index, off = u2(data, off)
        desc_index, off = u2(data, off)
        attr_count, off = u2(data, off)
        attrs, off = skip_attributes(data, off, attr_count)
        fields.append({
            "info": info,
            "access": field_access,
            "name_index": name_index,
            "desc_index": desc_index,
            "name": utf8(entries, name_index),
            "descriptor": utf8(entries, desc_index),
            "attrs": attrs,
        })
    method_count, off = u2(data, off)
    methods = []
    for _ in range(method_count):
        info = off
        method_access, off = u2(data, off)
        name_index, off = u2(data, off)
        desc_index, off = u2(data, off)
        attr_count, off = u2(data, off)
        attrs, off = skip_attributes(data, off, attr_count)
        methods.append({
            "info": info,
            "access": method_access,
            "name_index": name_index,
            "desc_index": desc_index,
            "name": utf8(entries, name_index),
            "descriptor": utf8(entries, desc_index),
            "attrs": attrs,
        })
    return this_class, fields, methods


def class_name(entries, class_index):
    item = entries[class_index]
    if item is None or item[0] != 7:
        raise ValueError(f"constant {class_index} is not Class")
    return utf8(entries, struct.unpack(">H", item[1])[0])


def fieldref_parts(entries, index):
    item = entries[index]
    if item is None or item[0] != 9:
        raise ValueError(f"constant {index} is not Fieldref")
    class_index, nat_index = struct.unpack(">HH", item[1])
    nat = entries[nat_index]
    if nat is None or nat[0] != 12:
        raise ValueError(f"Fieldref {index} has non-NameAndType {nat_index}")
    name_index, desc_index = struct.unpack(">HH", nat[1])
    return class_index, nat_index, name_index, desc_index


def method_code_info(data, entries, method):
    code_attrs = [a for a in method["attrs"] if utf8(entries, a[0]) == "Code"]
    if len(code_attrs) != 1:
        raise ValueError(f"method {method['name']} has {len(code_attrs)} Code attrs")
    _, attr_data, _ = code_attrs[0]
    max_stack, cursor = u2(data, attr_data)
    max_locals, cursor = u2(data, cursor)
    code_length, cursor = u4(data, cursor)
    code = bytes(data[cursor:cursor + code_length])
    return {
        "method": method["name"],
        "descriptor": method["descriptor"],
        "code_attribute_offset": attr_data,
        "code_length": code_length,
        "code_sha256": sha(code),
        "max_stack": max_stack,
        "max_locals": max_locals,
    }


def main():
    if len(sys.argv) != 3:
        raise SystemExit(f"usage: {sys.argv[0]} INPUT.class OUTPUT.class")
    source = Path(sys.argv[1])
    output = Path(sys.argv[2])
    data = bytearray(source.read_bytes())
    original_sha = sha(data)
    entries, cp_end = parse_cp(data)
    this_class, fields, methods = class_layout(data, entries)
    owner = class_name(entries, this_class)
    field_by_name = {field["name"]: field for field in fields}
    missing = sorted(set(TARGETS) - set(field_by_name))
    if missing:
        raise ValueError(f"missing fields: {missing}")
    for field_name, descriptor in TARGETS.items():
        if field_by_name[field_name]["descriptor"] != "I":
            raise ValueError(f"{field_name} source descriptor is not I")

    descriptor_indices = {}
    for field_name, descriptor in TARGETS.items():
        data, entries, cp_end, new_index = add_utf8(data, entries, cp_end, descriptor)
        descriptor_indices[field_name] = new_index
    entries, _ = parse_cp(data)
    this_class, fields, methods = class_layout(data, entries)
    field_by_name = {field["name"]: field for field in fields}
    report = {
        "source": str(source),
        "output": str(output),
        "owner": owner,
        "source_sha256": original_sha,
        "fields": [],
        "methods": [method_code_info(data, entries, method) for method in methods],
    }
    method_names_by_field = {
        "byteField": ["setByte", "setByteProduced", "setByteOn", "setByteProducedOn"],
        "charField": ["setChar", "setCharProduced", "setCharOn", "setCharProducedOn"],
        "shortField": ["setShort", "setShortProduced", "setShortOn", "setShortProducedOn"],
        "booleanField": ["setBoolean", "setBooleanProduced", "setBooleanOn", "setBooleanProducedOn"],
        "staticByteField": ["setStaticByte", "setStaticByteProduced"],
        "staticCharField": ["setStaticChar", "setStaticCharProduced"],
        "staticShortField": ["setStaticShort", "setStaticShortProduced"],
        "staticBooleanField": ["setStaticBoolean", "setStaticBooleanProduced"],
    }
    for field_name, descriptor in TARGETS.items():
        field = field_by_name[field_name]
        field_info_desc_offset = field["info"] + 4
        old_field_desc_index = struct.unpack_from(">H", data, field_info_desc_offset)[0]
        put_u2(data, field_info_desc_offset, descriptor_indices[field_name])
        matches = []
        for index, item in enumerate(entries):
            if item is None or item[0] != 9:
                continue
            class_index, nat_index, name_index, desc_index = fieldref_parts(entries, index)
            if class_name(entries, class_index) == owner and utf8(entries, name_index) == field_name and utf8(entries, desc_index) == "I":
                matches.append((index, nat_index, desc_index))
        if len(matches) != 1:
            raise ValueError(f"{field_name} expected one matching Fieldref, found {len(matches)}")
        fieldref_index, nat_index, old_nat_desc_index = matches[0]
        nat_item = entries[nat_index]
        nat_desc_offset = nat_item[2] + 1 + 2
        put_u2(data, nat_desc_offset, descriptor_indices[field_name])
        entries, _ = parse_cp(data)
        report["fields"].append({
            "name": field_name,
            "source_descriptor": "I",
            "patched_descriptor": descriptor,
            "field_info_offset": field["info"],
            "field_info_descriptor_index_before": old_field_desc_index,
            "field_info_descriptor_index_after": descriptor_indices[field_name],
            "fieldref_cp_index": fieldref_index,
            "name_and_type_cp_index": nat_index,
            "name_and_type_descriptor_index_before": old_nat_desc_index,
            "name_and_type_descriptor_index_after": descriptor_indices[field_name],
            "writer_methods": method_names_by_field[field_name],
        })
    # Re-read Code after every constant-pool edit; bytecode must be unchanged.
    _, _, after_methods = class_layout(data, entries)
    after_by_key = {(method["name"], method["descriptor"]): method for method in after_methods}
    for method_info in report["methods"]:
        key = (method_info["method"], method_info["descriptor"])
        after = method_code_info(data, entries, after_by_key[key])
        method_info["code_sha256_after"] = after["code_sha256"]
        if method_info["code_sha256_after"] != method_info["code_sha256"]:
            raise ValueError(f"Code changed unexpectedly for {key}")
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_bytes(data)
    report["patched_sha256"] = sha(data)
    report["constant_pool_count"] = struct.unpack_from(">H", data, 8)[0]
    report["field_count"] = len(fields)
    report["method_count"] = len(methods)
    output.with_suffix(".patch.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
