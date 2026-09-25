#!/usr/bin/env python3
"""Patch only the method descriptors and one iastore in each selected Code body."""
import hashlib
import json
import struct
import sys
from pathlib import Path

TARGETS = {
    "storeByte": ("([BII)V", 0x54, "bastore", "[B"),
    "storeChar": ("([CII)V", 0x55, "castore", "[C"),
    "storeShort": ("([SII)V", 0x56, "sastore", "[S"),
    "storeByteProduced": ("([BIIZ)V", 0x54, "bastore", "[B"),
    "storeCharProduced": ("([CIIZ)V", 0x55, "castore", "[C"),
    "storeShortProduced": ("([SIIZ)V", 0x56, "sastore", "[S"),
}


def u1(data, off):
    return data[off], off + 1


def u2(data, off):
    return struct.unpack_from(">H", data, off)[0], off + 2


def u4(data, off):
    return struct.unpack_from(">I", data, off)[0], off + 4


def put_u2(data, off, value):
    struct.pack_into(">H", data, off, value)


def parse_cp(data):
    count = struct.unpack_from(">H", data, 8)[0]
    entries = [None] * count
    off = 10
    for index in range(1, count):
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
    return entries, off


def cp_utf8(entries, index):
    item = entries[index]
    if item is None or item[0] != 1:
        raise ValueError(f"cp {index} is not Utf8")
    return item[1].decode("utf-8")


def cp_add_utf8(data, entries, cp_end, value):
    for index, item in enumerate(entries):
        if item is not None and item[0] == 1 and item[1].decode("utf-8") == value:
            return data, entries, cp_end, index
    encoded = value.encode("utf-8")
    if len(encoded) > 0xFFFF:
        raise ValueError("Utf8 constant too long")
    index = len(entries)
    entries.append((1, encoded, None))
    addition = bytes([1]) + struct.pack(">H", len(encoded)) + encoded
    old_count = struct.unpack_from(">H", data, 8)[0]
    if old_count >= 0xFFFF:
        raise ValueError("constant pool full")
    new_data = data[:8] + struct.pack(">H", old_count + 1) + data[10:cp_end] + addition + data[cp_end:]
    return new_data, entries, cp_end + len(addition), index


def skip_attributes(data, off, count):
    attrs = []
    for _ in range(count):
        name_index, off = u2(data, off)
        length, off = u4(data, off)
        attrs.append((name_index, off, length))
        off += length
    return attrs, off


def find_methods(data, entries):
    off = 8
    _, off = u2(data, off)  # constant_pool_count
    _, off = u2(data, off)  # first cp entry is at 10; this line is intentionally not used
    # Recompute from parsed pool because long entries make manual stepping error-prone.
    _, off = parse_cp(data)
    _, off = u2(data, off)  # access_flags
    _, off = u2(data, off)  # this_class
    _, off = u2(data, off)  # super_class
    interface_count, off = u2(data, off)
    off += 2 * interface_count
    field_count, off = u2(data, off)
    for _ in range(field_count):
        off += 6
        attr_count, off = u2(data, off)
        _, off = skip_attributes(data, off, attr_count)
    method_count, off = u2(data, off)
    methods = []
    for _ in range(method_count):
        info = off
        access, off = u2(data, off)
        name_index, off = u2(data, off)
        desc_index, off = u2(data, off)
        attr_count, off = u2(data, off)
        attrs, off = skip_attributes(data, off, attr_count)
        methods.append({
            "info": info,
            "access": access,
            "name_index": name_index,
            "desc_index": desc_index,
            "name": cp_utf8(entries, name_index),
            "descriptor": cp_utf8(entries, desc_index),
            "attrs": attrs,
        })
    return methods


def sha(data):
    return hashlib.sha256(data).hexdigest()


def main():
    if len(sys.argv) == 4 and sys.argv[1] == "--boolean-boundary":
        patch_boolean_boundary(Path(sys.argv[2]), Path(sys.argv[3]))
        return
    if len(sys.argv) == 4 and sys.argv[1] == "--boolean-operand-boundary":
        patch_boolean_operand_boundary(Path(sys.argv[2]), Path(sys.argv[3]))
        return
    if len(sys.argv) != 3:
        raise SystemExit(
            f"usage: {sys.argv[0]} INPUT.class OUTPUT.class\n"
            f"       {sys.argv[0]} --boolean-boundary INPUT.patched.class OUTPUT.class\n"
            f"       {sys.argv[0]} --boolean-operand-boundary INPUT.class OUTPUT.class"
        )
    source = Path(sys.argv[1])
    output = Path(sys.argv[2])
    data = bytearray(source.read_bytes())
    original_sha = sha(data)
    entries, cp_end = parse_cp(data)
    descriptor_indices = {}
    for method_name, (descriptor, _, _, _) in TARGETS.items():
        data, entries, cp_end, descriptor_indices[descriptor] = cp_add_utf8(
            data, entries, cp_end, descriptor
        )
    methods = find_methods(data, entries)
    selected = {}
    for method in methods:
        if method["name"] in TARGETS:
            if method["name"] in selected:
                raise ValueError(f"duplicate target method {method['name']}")
            selected[method["name"]] = method
    missing = sorted(set(TARGETS) - set(selected))
    if missing:
        raise ValueError(f"missing target methods: {missing}")
    report = {
        "source": str(source),
        "output": str(output),
        "source_sha256": original_sha,
        "methods": [],
    }
    for name, (new_descriptor, replacement, mnemonic, array_type) in TARGETS.items():
        method = selected[name]
        if method["descriptor"] != "([III)V" and method["descriptor"] != "([IIIZ)V":
            raise ValueError(f"{name} unexpected source descriptor {method['descriptor']}")
        desc_index_offset = method["info"] + 4
        old_desc_index = struct.unpack_from(">H", data, desc_index_offset)[0]
        put_u2(data, desc_index_offset, descriptor_indices[new_descriptor])
        code_attrs = [a for a in method["attrs"] if cp_utf8(entries, a[0]) == "Code"]
        if len(code_attrs) != 1:
            raise ValueError(f"{name} has {len(code_attrs)} Code attrs")
        _, attr_data, attr_len = code_attrs[0]
        max_stack, cursor = u2(data, attr_data)
        max_locals, cursor = u2(data, cursor)
        code_length, cursor = u4(data, cursor)
        code_start = cursor
        code_end = code_start + code_length
        code = data[code_start:code_end]
        offsets = [index for index, opcode in enumerate(code) if opcode == 0x4F]
        if len(offsets) != 1:
            raise ValueError(f"{name} expected one iastore, found {len(offsets)}")
        relative = offsets[0]
        before_code_sha = sha(bytes(code))
        before_opcode = code[relative]
        data[code_start + relative] = replacement
        after_code_sha = sha(bytes(data[code_start:code_end]))
        report["methods"].append({
            "name": name,
            "source_descriptor": method["descriptor"],
            "source_descriptor_index": old_desc_index,
            "patched_descriptor": new_descriptor,
            "array_type": array_type,
            "code_attribute_offset": attr_data,
            "code_length": code_length,
            "opcode_offset_in_code": relative,
            "opcode_before": f"0x{before_opcode:02x} iastore",
            "opcode_after": f"0x{replacement:02x} {mnemonic}",
            "code_sha256_before": before_code_sha,
            "code_sha256_after": after_code_sha,
            "max_stack": max_stack,
            "max_locals": max_locals,
        })
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_bytes(data)
    report["patched_sha256"] = sha(data)
    report["constant_pool_count"] = struct.unpack_from(">H", data, 8)[0]
    report["method_count"] = len(methods)
    output.with_suffix(".patch.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report, indent=2))


def patch_boolean_boundary(source, output):
    """Change the B sample's array proof to Z while retaining legal bastore."""
    data = bytearray(source.read_bytes())
    entries, cp_end = parse_cp(data)
    data, entries, cp_end, descriptor_index = cp_add_utf8(
        data, entries, cp_end, "([ZII)V"
    )
    methods = find_methods(data, entries)
    selected = [method for method in methods if method["name"] == "storeByte"]
    if len(selected) != 1 or selected[0]["descriptor"] != "([BII)V":
        raise ValueError("boolean boundary expects patched storeByte ([BII)V")
    method = selected[0]
    put_u2(data, method["info"] + 4, descriptor_index)
    method = next(method for method in find_methods(data, entries)
                  if method["name"] == "storeByte")
    code_attrs = [attr for attr in method["attrs"]
                  if cp_utf8(entries, attr[0]) == "Code"]
    if len(code_attrs) != 1:
        raise ValueError("storeByte must have exactly one Code attribute")
    _, attr_data, _ = code_attrs[0]
    _, cursor = u2(data, attr_data)
    _, cursor = u2(data, cursor)
    code_length, cursor = u4(data, cursor)
    code_end = cursor + code_length
    code = data[cursor:code_end]
    offsets = [index for index, opcode in enumerate(code) if opcode == 0x54]
    if len(offsets) != 1:
        raise ValueError(f"storeByte expected one bastore, found {len(offsets)}")
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_bytes(data)
    report = {
        "source_sha256": sha(source.read_bytes()),
        "output_sha256": sha(data),
        "method": "storeByte",
        "source_descriptor": "([BII)V",
        "patched_descriptor": "([ZII)V",
        "opcode": "0x54 bastore (unchanged)",
        "code_length": code_length,
        "opcode_offset_in_code": offsets[0],
    }
    output.with_suffix(".patch.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report, indent=2))


def patch_boolean_operand_boundary(source, output):
    """Patch a boolean[] parameter to byte[] while retaining bastore and boolean input."""
    data = bytearray(source.read_bytes())
    entries, cp_end = parse_cp(data)
    data, entries, cp_end, descriptor_index = cp_add_utf8(
        data, entries, cp_end, "([BIZ)V"
    )
    methods = find_methods(data, entries)
    selected = [method for method in methods
                if method["name"] == "storeByteOperand"]
    if len(selected) != 1 or selected[0]["descriptor"] != "([ZIZ)V":
        raise ValueError("boolean operand expects storeByteOperand ([ZIZ)V")
    method = selected[0]
    put_u2(data, method["info"] + 4, descriptor_index)
    method = next(method for method in find_methods(data, entries)
                  if method["name"] == "storeByteOperand")
    code_attrs = [attr for attr in method["attrs"]
                  if cp_utf8(entries, attr[0]) == "Code"]
    if len(code_attrs) != 1:
        raise ValueError("storeByteOperand must have exactly one Code attribute")
    _, attr_data, _ = code_attrs[0]
    _, cursor = u2(data, attr_data)
    _, cursor = u2(data, cursor)
    code_length, cursor = u4(data, cursor)
    code_end = cursor + code_length
    code = data[cursor:code_end]
    offsets = [index for index, opcode in enumerate(code) if opcode == 0x54]
    if len(offsets) != 1:
        raise ValueError(f"storeByteOperand expected one bastore, found {len(offsets)}")
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_bytes(data)
    report = {
        "source_sha256": sha(source.read_bytes()),
        "output_sha256": sha(data),
        "method": "storeByteOperand",
        "source_descriptor": "([ZIZ)V",
        "patched_descriptor": "([BIZ)V",
        "opcode": "0x54 bastore (unchanged)",
        "code_length": code_length,
        "opcode_offset_in_code": offsets[0],
    }
    output.with_suffix(".patch.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
