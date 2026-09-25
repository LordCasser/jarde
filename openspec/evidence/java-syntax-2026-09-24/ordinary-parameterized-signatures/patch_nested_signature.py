#!/usr/bin/env python3
"""Replace only NestedMissingSignature.echo's generic argument in its Signature attribute."""
import pathlib
import struct
import sys


def u2(data, offset):
    return struct.unpack_from(">H", data, offset)[0]


def u4(data, offset):
    return struct.unpack_from(">I", data, offset)[0]


def main(source, destination):
    data = bytearray(pathlib.Path(source).read_bytes())
    if u4(data, 0) != 0xCAFEBABE:
        raise SystemExit("not a class file")
    count = u2(data, 8)
    offset = 10
    utf8 = {}
    index = 1
    while index < count:
        tag = data[offset]
        offset += 1
        if tag == 1:
            length = u2(data, offset)
            utf8[index] = (offset + 2, length, bytes(data[offset + 2 : offset + 2 + length]))
            offset += 2 + length
        elif tag in (3, 4, 9, 10, 11, 12, 17, 18):
            offset += 4
        elif tag in (5, 6):
            offset += 8
            index += 1
        elif tag == 7 or tag == 8 or tag == 16 or tag == 19 or tag == 20:
            offset += 2
        elif tag == 15:
            offset += 3
        else:
            raise SystemExit(f"unsupported constant-pool tag {tag}")
        index += 1

    def skip_attributes(pos):
        attribute_count = u2(data, pos)
        pos += 2
        found = []
        for _ in range(attribute_count):
            name_index = u2(data, pos)
            length = u4(data, pos + 2)
            payload = pos + 6
            if utf8[name_index][2] == b"Signature":
                found.append((payload, length))
            pos = payload + length
        return pos, found

    # Access flags, this class, super class, interfaces.
    offset += 6
    interfaces = u2(data, offset)
    offset += 2 + interfaces * 2
    # Fields.
    fields = u2(data, offset)
    offset += 2
    for _ in range(fields):
        offset += 6
        offset, _ = skip_attributes(offset)
    # Methods.
    methods = u2(data, offset)
    offset += 2
    matches = []
    for _ in range(methods):
        name_index = u2(data, offset + 2)
        descriptor_index = u2(data, offset + 4)
        name = utf8[name_index][2]
        descriptor = utf8[descriptor_index][2]
        offset += 6
        offset, signatures = skip_attributes(offset)
        if name == b"echo" and descriptor == b"(Ljava/util/List;)Ljava/util/List;":
            if len(signatures) != 1 or signatures[0][1] != 2:
                raise SystemExit("echo must have one two-byte Signature attribute")
            matches.append(signatures[0][0])
    if len(matches) != 1:
        raise SystemExit(f"expected one echo Signature attribute, found {len(matches)}")
    signature_index = u2(data, matches[0])
    string_offset, length, raw = utf8[signature_index]
    before = b"(Ljava/util/List<Lprobe/Payload;>;)Ljava/util/List<Lprobe/Payload;>;"
    after = b"(Ljava/util/List<Lprobe/Missing;>;)Ljava/util/List<Lprobe/Missing;>;"
    if raw != before or len(before) != len(after) or length != len(before):
        raise SystemExit(f"unexpected echo Signature: {raw!r}")
    data[string_offset : string_offset + length] = after
    pathlib.Path(destination).write_bytes(data)


if __name__ == "__main__":
    if len(sys.argv) != 3:
        raise SystemExit("usage: patch_nested_signature.py input.class output.class")
    main(sys.argv[1], sys.argv[2])
