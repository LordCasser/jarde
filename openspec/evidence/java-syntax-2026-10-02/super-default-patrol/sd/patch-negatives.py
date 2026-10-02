#!/usr/bin/env python3
"""Patch the two frozen negative variant classes in place (indices only, no length changes).

Both negatives are shapes `javac --release 8` refuses to emit, so each starts from its legal
neighbour and repoints one constant-pool index:

- `SDIndirect$Use`: the header `interfaces` slot moves from the direct interface `SDIndirect$SDA`
  to `SDIndirect$SDM` (which `extends SDA`), so the `invokespecial InterfaceMethod SDA.name`
  qualifier is now inherited through the extends chain only. The class stays a subtype of `SDA`
  through `SDM`, so the JVM still links and runs it (`SA`).
- `SDAbstract$Use`: the `InterfaceMethodref` of the qualified call moves from `SDAbstract$SDB`
  (default `name`) to the direct interface `SDAbstract$SDC` (abstract `name`), so the target is a
  direct superinterface whose member-table entry is abstract. The method still resolves, so the
  class links under `-Xverify:all` (calling `name` raises AbstractMethodError).

Usage: python3 patch-negatives.py CLASSES_DIR
"""

import struct
import sys
from pathlib import Path

CONSTANT_Utf8 = 1
CONSTANT_Class = 7
CONSTANT_InterfaceMethodref = 11


def parse_pool(data):
    """Return (entries, offsets) where entries[i] is (tag, payload) for 1-based indices."""
    count = struct.unpack_from(">H", data, 8)[0]
    entries, offsets, i, offset = {}, {}, 1, 10
    while i < count:
        tag = data[offset]
        if tag == CONSTANT_Utf8:
            length = struct.unpack_from(">H", data, offset + 1)[0]
            payload = data[offset + 3 : offset + 3 + length]
            size = 3 + length
        elif tag in (7, 8, 16, 19, 20):
            payload = struct.unpack_from(">H", data, offset + 1)[0]
            size = 3
        elif tag in (9, 10, 11, 12, 17, 18):
            payload = struct.unpack_from(">HH", data, offset + 1)
            size = 5
        elif tag == 15:
            payload = (data[offset + 1], struct.unpack_from(">H", data, offset + 2)[0])
            size = 4
        elif tag in (3, 4):
            payload = struct.unpack_from(">I", data, offset + 1)[0]
            size = 5
        elif tag in (5, 6):
            payload = None
            size = 9
        else:
            raise ValueError(f"unknown pool tag {tag} at byte {offset}")
        entries[i] = (tag, payload)
        offsets[i] = (offset, offset + size)
        i += 2 if tag in (5, 6) else 1
        offset += size
    return entries, offsets


def utf8_name(entries, index):
    tag, payload = entries[index]
    assert tag == CONSTANT_Utf8, (index, tag)
    return payload.decode("utf-8")


def class_name(entries, class_index):
    tag, payload = entries[class_index]
    assert tag == CONSTANT_Class, (class_index, tag)
    return utf8_name(entries, payload)


def class_index_of(entries, name):
    matches = [
        index
        for index, (tag, _) in entries.items()
        if tag == CONSTANT_Class and class_name(entries, index) == name
    ]
    assert len(matches) == 1, (name, matches)
    return matches[0]


def patch_interfaces_slot(data, entries, offsets, from_name, to_name):
    """Repoint the header `interfaces` entry naming from_name to to_name's class index."""
    target = class_index_of(entries, to_name)
    tail = offsets[max(entries)][1]
    access, this, super_class, count = struct.unpack_from(">HHHH", data, tail)
    for slot in range(count):
        at = tail + 8 + 2 * slot
        index = struct.unpack_from(">H", data, at)[0]
        if class_name(entries, index) == from_name:
            data[at : at + 2] = struct.pack(">H", target)
            return
    raise AssertionError(f"no {from_name} in interfaces of {class_name(entries, this)}")


def patch_interface_methodref_owner(data, entries, offsets, from_owner, name, descriptor, to_owner):
    """Repoint the one InterfaceMethodref (from_owner.name.descriptor) to to_owner."""
    target = class_index_of(entries, to_owner)
    hits = []
    for index, (tag, payload) in entries.items():
        if tag != CONSTANT_InterfaceMethodref:
            continue
        if class_name(entries, payload[0]) != from_owner:
            continue
        name_and_type = entries[payload[1]]
        if utf8_name(entries, name_and_type[1][0]) != name:
            continue
        if utf8_name(entries, name_and_type[1][1]) != descriptor:
            continue
        hits.append(index)
    assert len(hits) == 1, (from_owner, name, hits)
    offset = offsets[hits[0]][0]
    data[offset + 1 : offset + 3] = struct.pack(">H", target)


def main():
    classes = Path(sys.argv[1])
    for file, action in (
        ("SDIndirect$Use.class", "interfaces"),
        ("SDAbstract$Use.class", "methodref"),
    ):
        path = classes / file
        data = bytearray(path.read_bytes())
        entries, offsets = parse_pool(data)
        if action == "interfaces":
            patch_interfaces_slot(
                data, entries, offsets, "SDIndirect$SDA", "SDIndirect$SDM"
            )
        else:
            patch_interface_methodref_owner(
                data,
                entries,
                offsets,
                "SDAbstract$SDB",
                "name",
                "()Ljava/lang/String;",
                "SDAbstract$SDC",
            )
        path.write_bytes(bytes(data))
        print(f"patched {file}")


if __name__ == "__main__":
    main()
