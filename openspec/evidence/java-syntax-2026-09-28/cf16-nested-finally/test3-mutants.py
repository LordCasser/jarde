#!/usr/bin/env python3
"""Freeze verifier-valid test3 near misses from the pinned Java 8 class."""

from pathlib import Path
from struct import pack, unpack_from

HERE = Path(__file__).resolve().parent
SOURCE = HERE / "TestTryCatchFinally12$TestCls.class"


def u2(data, at):
    return unpack_from(">H", data, at)[0]


def u4(data, at):
    return unpack_from(">I", data, at)[0]


def constant_pool_end(data):
    count = u2(data, 8)
    at = 10
    utf8 = {}
    index = 1
    while index < count:
        tag = data[at]
        at += 1
        if tag == 1:
            size = u2(data, at)
            at += 2
            utf8[index] = bytes(data[at : at + size]).decode()
            at += size
        elif tag in (3, 4, 9, 10, 11, 12, 17, 18):
            at += 4
        elif tag in (5, 6):
            at += 8
            index += 1
        elif tag in (7, 8, 16, 19, 20):
            at += 2
        elif tag == 15:
            at += 3
        else:
            raise ValueError(f"unknown constant pool tag {tag}")
        index += 1
    return at, utf8


def test3_code(data):
    at, utf8 = constant_pool_end(data)
    at += 6  # access, this, super
    interfaces = u2(data, at)
    at += 2 + 2 * interfaces
    fields = u2(data, at)
    at += 2
    for _ in range(fields):
        at += 6
        attributes = u2(data, at)
        at += 2
        for _ in range(attributes):
            at += 6 + u4(data, at + 2)
    methods = u2(data, at)
    at += 2
    for _ in range(methods):
        name = utf8[u2(data, at + 2)]
        attributes = u2(data, at + 6)
        at += 8
        for _ in range(attributes):
            attr_name = utf8[u2(data, at)]
            length = u4(data, at + 2)
            if name == "test3" and attr_name == "Code":
                code = at + 14
                table = code + u4(data, at + 10)
                return code, table
            at += 6 + length
    raise ValueError("test3 Code not found")


def add_second_field(data):
    at, _ = constant_pool_end(data)
    at += 6
    interfaces = u2(data, at)
    at += 2 + 2 * interfaces
    fields = u2(data, at)
    data[at : at + 2] = pack(">H", fields + 1)
    at += 2
    for _ in range(fields):
        at += 6
        attributes = u2(data, at)
        at += 2
        for _ in range(attributes):
            at += 6 + u4(data, at + 2)
    data[at:at] = pack(">HHHH", 0x0002, 99, 18, 0)


def save(name, edit, extra=b"", second_field=False):
    data = bytearray(SOURCE.read_bytes())
    if extra:
        end, _ = constant_pool_end(data)
        count = u2(data, 8)
        data[8:10] = pack(">H", count + 3)
        data[end:end] = extra
    if second_field:
        add_second_field(data)
    code, table = test3_code(data)
    assert data[code + 9] == 0x12 and data[code + 10] == 29
    assert data[code + 11] == 0xB6 and u2(data, code + 12) == 21
    assert data[code + 14] == 0x57
    assert u2(data, table) == 3
    edit(data, code, table)
    (HERE / name).write_bytes(data)


save("Test3DifferentConstant.class", lambda d, c, t: d.__setitem__(c + 10, 19))


def same_other_constant(data, code, table):
    for offset in (10, 34, 48):
        assert data[code + offset] == 29
        data[code + offset] = 19


save(
    "Test3MatchingOtherConstant.class",
    same_other_constant,
)
save("Test3StoredResult.class", lambda d, c, t: d.__setitem__(c + 14, 0x4D))
save("Test3WidenedRow.class", lambda d, c, t: d.__setitem__(slice(t + 12, t + 14), pack(">H", 15)))

# An existing StringBuilder overload accepts the same String value, returns the same type,
# and executes successfully. Only one copy names it; the three invoked members differ.
descriptor = b"(Ljava/lang/CharSequence;)Ljava/lang/StringBuilder;"
extra = b"\x01" + pack(">H", len(descriptor)) + descriptor
extra += b"\x0c" + pack(">HH", 25, 99)
extra += b"\x0a" + pack(">HH", 22, 100)
save(
    "Test3DifferentTarget.class",
    lambda d, c, t: d.__setitem__(slice(c + 12, c + 14), pack(">H", 101)),
    extra,
)

field_name = b"sb2"
field_extra = b"\x01" + pack(">H", len(field_name)) + field_name
field_extra += b"\x0c" + pack(">HH", 99, 18)
field_extra += b"\x09" + pack(">HH", 8, 100)
save(
    "Test3DifferentField.class",
    lambda d, c, t: d.__setitem__(slice(c + 7, c + 9), pack(">H", 101)),
    field_extra,
    second_field=True,
)
