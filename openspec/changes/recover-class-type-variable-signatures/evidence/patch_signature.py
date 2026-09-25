#!/usr/bin/env python3
"""Replace one class- or method-level Signature constant in a class file."""

import pathlib
import struct
import sys


def read_u2(data, offset):
    return struct.unpack_from(">H", data, offset)[0]


def read_u4(data, offset):
    return struct.unpack_from(">I", data, offset)[0]


def main(source, destination, selector, old, new):
    data = bytearray(pathlib.Path(source).read_bytes())
    if read_u4(data, 0) != 0xCAFEBABE:
        raise SystemExit("not a class file")
    cp_count = read_u2(data, 8)
    offset = 10
    utf8 = {}
    index = 1
    while index < cp_count:
        tag = data[offset]
        offset += 1
        if tag == 1:
            length = read_u2(data, offset)
            utf8[index] = (offset + 2, length, bytes(data[offset + 2 : offset + 2 + length]))
            offset += 2 + length
        elif tag in (3, 4, 9, 10, 11, 12, 17, 18):
            offset += 4
        elif tag in (5, 6):
            offset += 8
            index += 1
        elif tag in (7, 8, 16, 19, 20):
            offset += 2
        elif tag == 15:
            offset += 3
        else:
            raise SystemExit(f"unsupported constant-pool tag {tag}")
        index += 1

    def attributes(pos):
        count = read_u2(data, pos)
        pos += 2
        matches = []
        for _ in range(count):
            name_index = read_u2(data, pos)
            length = read_u4(data, pos + 2)
            payload = pos + 6
            if utf8[name_index][2] == b"Signature":
                matches.append(payload)
            pos = payload + length
        return pos, matches

    # Skip class header and interfaces.
    offset += 6
    interface_count = read_u2(data, offset)
    offset += 2 + interface_count * 2
    field_count = read_u2(data, offset)
    offset += 2
    for _ in range(field_count):
        offset += 6
        offset, _ = attributes(offset)

    target_payload = None
    method_name = selector.removeprefix("method:").encode("ascii") if selector.startswith("method:") else None
    method_count = read_u2(data, offset)
    offset += 2
    for _ in range(method_count):
        name_index = read_u2(data, offset + 2)
        name = utf8[name_index][2]
        offset += 6
        offset, signatures = attributes(offset)
        if name == method_name and method_name is not None:
            if len(signatures) != 1:
                raise SystemExit(f"expected one Signature for method {method_name!r}")
            target_payload = signatures[0]

    if selector == "class":
        offset, signatures = attributes(offset)
        if len(signatures) != 1:
            raise SystemExit("expected one class Signature")
        target_payload = signatures[0]
    elif not selector.startswith("method:"):
        raise SystemExit("selector must be 'class' or 'method:<name>'")

    if target_payload is None:
        raise SystemExit(f"Signature selector not found: {selector}")
    cp_index = read_u2(data, target_payload)
    string_offset, old_length, current = utf8[cp_index]
    old_bytes = old.encode("ascii")
    new_bytes = new.encode("ascii")
    if current != old_bytes:
        raise SystemExit(f"Signature did not match expected value: {current!r}")
    data[string_offset - 2 : string_offset] = struct.pack(">H", len(new_bytes))
    data[string_offset : string_offset + old_length] = new_bytes
    pathlib.Path(destination).write_bytes(data)


if __name__ == "__main__":
    if len(sys.argv) != 6:
        raise SystemExit("usage: patch_signature.py input.class output.class class|method:NAME OLD NEW")
    main(*sys.argv[1:])
