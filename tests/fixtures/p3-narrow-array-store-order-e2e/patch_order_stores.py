#!/usr/bin/env python3
"""Patch the three source int[] stores to verifier-valid B/C/S stores in place."""
import json
import struct
import sys
from pathlib import Path


def u2(data, off):
    return struct.unpack_from(">H", data, off)[0], off + 2


def u4(data, off):
    return struct.unpack_from(">I", data, off)[0], off + 4


def cp(data):
    count = struct.unpack_from(">H", data, 8)[0]
    entries = [None] * count
    off = 10
    index = 1
    while index < count:
        tag = data[off]
        start = off
        off += 1
        if tag == 1:
            length, off = u2(data, off)
            raw = bytes(data[off:off + length])
            off += length
            entries[index] = (tag, raw, start)
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
            raise ValueError(f"unknown constant-pool tag {tag}")
        index += 1
    return entries, off


def utf(entries, index):
    item = entries[index]
    if item is None or item[0] != 1:
        raise ValueError(f"constant {index} is not Utf8")
    return item[1].decode("utf-8")


def attributes(data, off, count):
    result = []
    for _ in range(count):
        name, off = u2(data, off)
        length, off = u4(data, off)
        result.append((name, off, length))
        off += length
    return result, off


def methods(data, entries, cp_end):
    off = cp_end + 6
    interfaces, off = u2(data, off)
    off += interfaces * 2
    fields, off = u2(data, off)
    for _ in range(fields):
        off += 6
        count, off = u2(data, off)
        _, off = attributes(data, off, count)
    count, off = u2(data, off)
    result = []
    for _ in range(count):
        info = off
        _, off = u2(data, off)
        name, off = u2(data, off)
        descriptor, off = u2(data, off)
        attr_count, off = u2(data, off)
        attrs, off = attributes(data, off, attr_count)
        result.append({"info": info, "name": utf(entries, name),
                       "descriptor": utf(entries, descriptor), "attrs": attrs})
    return result


def main(source, output):
    data = bytearray(Path(source).read_bytes())
    entries, cp_end = cp(data)
    added = {}
    for value in ("([BIIZZZ)V", "([CIIZZZ)V", "([SIIZZZ)V",
                  "([BZ)[B", "([CZ)[C", "([SZ)[S"):
        found = next((i for i, item in enumerate(entries)
                      if item is not None and item[0] == 1 and item[1].decode() == value), None)
        if found is None:
            encoded = value.encode()
            entry = b"\x01" + struct.pack(">H", len(encoded)) + encoded
            data[cp_end:cp_end] = entry
            struct.pack_into(">H", data, 8, len(entries) + 1)
            cp_end += len(entry)
            entries.append((1, encoded, None))
            found = len(entries) - 1
        added[value] = found

    names = {"storeByte": "([BIIZZZ)V", "storeChar": "([CIIZZZ)V",
             "storeShort": "([SIIZZZ)V"}
    helpers = {"byteArray": "([BZ)[B", "charArray": "([CZ)[C",
               "shortArray": "([SZ)[S"}

    # Make each target method declaration advertise its actual array type.
    selected = {}
    for method in methods(data, entries, cp_end):
        if method["name"] in names:
            selected[method["name"]] = method
            struct.pack_into(">H", data, method["info"] + 4, added[names[method["name"]]])
    if set(selected) != set(names):
        raise ValueError("expected all three store methods")

    # Retype just the matching helper invocations' Methodref NameAndType descriptors.
    for index, item in enumerate(entries):
        if item is None or item[0] not in (10, 11):
            continue
        owner, nat = struct.unpack(">HH", item[1])
        nat_item = entries[nat]
        if nat_item is None or nat_item[0] != 12:
            continue
        name_index, descriptor_index = struct.unpack(">HH", nat_item[1])
        helper_name = utf(entries, name_index)
        if helper_name in helpers:
            nat_start = nat_item[2]
            struct.pack_into(">H", data, nat_start + 3, added[helpers[helper_name]])

    # Change the sole iastore in each method, retaining its Code length and all offsets.
    report = []
    for name, opcode, mnemonic in (("storeByte", 0x54, "bastore"),
                                   ("storeChar", 0x55, "castore"),
                                   ("storeShort", 0x56, "sastore")):
        method = selected[name]
        code_attrs = [attr for attr in method["attrs"] if utf(entries, attr[0]) == "Code"]
        if len(code_attrs) != 1:
            raise ValueError(f"{name} must have one Code attribute")
        _, off, _ = code_attrs[0]
        _, off = u2(data, off)
        _, off = u2(data, off)
        length, off = u4(data, off)
        end = off + length
        found = [pos for pos in range(off, end) if data[pos] == 0x4f]
        if len(found) != 1:
            raise ValueError(f"{name}: expected one iastore, found {len(found)}")
        data[found[0]] = opcode
        report.append({"method": name, "opcode": mnemonic,
                       "code_length": length, "offset": found[0] - off})

    Path(output).write_bytes(data)
    Path(output + ".json").write_text(json.dumps(report, indent=2) + "\n")


if __name__ == "__main__":
    if len(sys.argv) != 3:
        raise SystemExit("usage: patch_order_stores.py INPUT.class OUTPUT.class")
    main(sys.argv[1], sys.argv[2])
