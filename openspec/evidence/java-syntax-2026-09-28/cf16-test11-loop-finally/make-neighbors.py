#!/usr/bin/env python3
"""Freeze verifier-valid, single-edit bytecode neighbors of the Java 8 probe."""

import hashlib
import struct
import sys
from pathlib import Path


def u2(data, offset):
    return struct.unpack_from(">H", data, offset)[0]


def u4(data, offset):
    return struct.unpack_from(">I", data, offset)[0]


def put_u2(data, offset, value):
    struct.pack_into(">H", data, offset, value)


def put_u4(data, offset, value):
    struct.pack_into(">I", data, offset, value)


raw = Path(sys.argv[1]).read_bytes()
out = Path(sys.argv[2])
out.mkdir(parents=True, exist_ok=True)
assert raw[:4] == b"\xca\xfe\xba\xbe" and u2(raw, 6) == 52
pool = [None] * u2(raw, 8)
at = 10
index = 1
while index < len(pool):
    tag = raw[at]
    at += 1
    if tag == 1:
        length = u2(raw, at)
        pool[index] = (tag, raw[at + 2:at + 2 + length].decode())
        at += 2 + length
    elif tag in (3, 4):
        at += 4
    elif tag in (5, 6):
        at += 8
        index += 1
    elif tag in (7, 8, 16, 19, 20):
        pool[index] = (tag, u2(raw, at))
        at += 2
    elif tag in (9, 10, 11, 12, 17, 18):
        pool[index] = (tag, u2(raw, at), u2(raw, at + 2))
        at += 4
    elif tag == 15:
        at += 3
    else:
        raise ValueError(f"unsupported constant tag {tag}")
    index += 1


def utf8(index):
    assert pool[index][0] == 1
    return pool[index][1]


def member_name(index):
    tag, _, name_type = pool[index]
    assert tag in (10, 11)
    _, name, descriptor = pool[name_type]
    return utf8(name), utf8(descriptor)


call3 = next(index for index, item in enumerate(pool) if item and item[0] == 10
             and member_name(index) == ("call3", "(Ljava/lang/Object;)V"))
at += 6  # class access, this, super
interfaces = u2(raw, at)
at += 2 + 2 * interfaces


def skip_members(data, offset):
    count = u2(data, offset)
    offset += 2
    for _ in range(count):
        attrs = u2(data, offset + 6)
        offset += 8
        for _ in range(attrs):
            offset += 6 + u4(data, offset + 2)
    return offset


at = skip_members(raw, at)  # fields
methods = u2(raw, at)
at += 2
code_at = row_at = None
for _ in range(methods):
    name = utf8(u2(raw, at + 2))
    descriptor = utf8(u2(raw, at + 4))
    attrs = u2(raw, at + 6)
    at += 8
    for _ in range(attrs):
        attr_name = utf8(u2(raw, at))
        length = u4(raw, at + 2)
        if name == "test" and descriptor == "(Ljava/util/List;)V" and attr_name == "Code":
            code_at = at + 6 + 8
            assert u4(raw, at + 6 + 4) == 80
            row_at = code_at + 80 + 2
            assert u2(raw, code_at + 80) == 2
        at += 6 + length
assert code_at is not None and row_at is not None
assert raw[code_at + 17] == raw[code_at + 55] == 0x99
assert raw[code_at + 70] in (0xb6, 0xb7)
assert raw[code_at + 76:code_at + 78] == b"\x19\x04"
assert (u2(raw, row_at), u2(raw, row_at + 2), u2(raw, row_at + 4)) == (0, 4, 38)
assert (u2(raw, row_at + 8), u2(raw, row_at + 10), u2(raw, row_at + 12)) == (38, 40, 38)


def save(name, edit):
    data = bytearray(raw)
    edit(data)
    (out / f"{name}.class").write_bytes(data)


if len(sys.argv) == 4 and sys.argv[3] == "different-list":
    save("different-list", lambda data: data.__setitem__(code_at + 40, 0x2a))
else:
    def handler_second_entry(data):
        data[code_at + 33:code_at + 35] = b"\x00\x1a"  # 32 -> 58
        data[code_at + 47] = 2
        data[code_at + 49] = data[code_at + 59] = 2
        data[code_at + 76:code_at + 78] = b"\x01\x00"
        frame = data.find(b"\xff\x00\x09\x00\x06\x07")
        assert frame >= 0 and data[frame + 11:frame + 19] == b"\x00\x00\x07\x00\x3b\x07\x00\x13"
        data[frame + 11:frame + 19] = b"\x07\x00\x13\x00\x00\x00"
        data[frame + 19:frame + 19] = b"\x09"  # @58, same locals as @48
        assert data[frame + 20:frame + 23] == b"\xfa\x00\x1b"
        data[frame + 22] = 17  # @76 is now relative to @58
        stackmap_header = data.find(b"\xfc\x00\x0b\x07\x00\x13\xfa\x00\x17") - 8
        assert stackmap_header >= 0 and u2(data, stackmap_header + 6) == 6
        put_u4(data, stackmap_header + 2, u4(data, stackmap_header + 2) - 1)
        put_u2(data, stackmap_header + 6, 7)
        put_u4(data, code_at - 12, u4(data, code_at - 12) - 1)
    save("handler-second-entry", handler_second_entry)
    save("different-target", lambda data: put_u2(data, code_at + 71, call3))
    save("different-loop-test", lambda data: data.__setitem__(code_at + 55, 0x9a))
    save("cleanup-self-protected", lambda data: put_u2(data, row_at + 10, 76))
    save("throwable-rewritten", lambda data: data.__setitem__(slice(code_at + 76, code_at + 78), b"\x01\x00"))
    save("extra-normal-exit", lambda data: data.__setitem__(slice(code_at + 33, code_at + 35), b"\x00\x2f"))
with (out / "SHA256SUMS").open("w") as record:
    for path in sorted(out.glob("*.class")):
        record.write(f"{hashlib.sha256(path.read_bytes()).hexdigest()}  {path.name}\n")
