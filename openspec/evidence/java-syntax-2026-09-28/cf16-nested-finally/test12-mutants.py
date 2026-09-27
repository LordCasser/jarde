#!/usr/bin/env python3
"""Freeze verifier-valid two-copy finally near misses from the pinned class."""

from pathlib import Path
from struct import pack, unpack_from

HERE = Path(__file__).resolve().parent
SOURCE = HERE / "TestTryCatchFinally12$TestCls.class"


def u2(data, at):
    return unpack_from(">H", data, at)[0]


def u4(data, at):
    return unpack_from(">I", data, at)[0]


def code_for(data, method):
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
            if name == method and attr_name == "Code":
                code = at + 14
                table = code + u4(data, at + 10)
                assert u2(data, table) == 2
                return code, table
            at += 6 + length
    raise ValueError(f"{method} Code not found")


def save(method, kind, edit):
    data = bytearray(SOURCE.read_bytes())
    code, table = code_for(data, method)
    edit(data, code, table)
    (HERE / f"{method.capitalize()}{kind}.class").write_bytes(data)


for method, normal_ldc, normal_start, normal_return, handler_load, outer_end in (
    ("test1", 33, 29, 55, 53, 29),
    ("test2", 23, 19, 45, 43, 19),
):
    def different_constant(data, code, table):
        assert data[code + normal_ldc : code + normal_ldc + 2] == bytes([0x12, 29])
        data[code + normal_ldc + 1] = 19  # existing String "-catch"

    def widened_row(data, code, table):
        assert u2(data, table + 12) == outer_end
        data[table + 12 : table + 14] = pack(">H", outer_end + 1)

    def bypass_cleanup(data, code, table):
        assert data[code + 5] == 0xA7
        data[code + 6 : code + 8] = pack(">h", normal_return - 5)

    def changed_rethrow(data, code, table):
        assert data[code + handler_load] == 0x2D  # aload_3
        data[code + handler_load] = 0x01  # aconst_null: verifier-valid, different Throwable

    save(method, "DifferentConstant", different_constant)
    save(method, "WidenedRow", widened_row)
    save(method, "BypassCleanup", bypass_cleanup)
    save(method, "ChangedRethrow", changed_rethrow)
