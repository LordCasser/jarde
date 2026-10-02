#!/usr/bin/env python3
"""Patch one exception-table entry of one method in a Java 8 class file.

Reads the classfile format just far enough to find the named method's Code
attribute and rewrite one exception_table row's start/end/handler pc fields.
Usage: patch_row.py <in.class> <out.class> <method-name> <row-index> <field> <value>
where <field> is start|end|handler and row-index counts from 0 in table order.
"""
import struct
import sys


def u2(data, at):
    return struct.unpack_from(">H", data, at)[0]


def skip_constant_pool(data):
    count = u2(data, 8)
    at = 10
    index = 1
    while index < count:
        tag = data[at]
        at += 1
        if tag == 1:
            at += 2 + u2(data, at)
        elif tag == 7:
            at += 2
        elif tag == 9 or tag == 10 or tag == 11 or tag == 3 or tag == 4 or tag == 12:
            at += 4
        elif tag == 8:
            at += 2
        elif tag == 5 or tag == 6:
            at += 8
            index += 1  # 8-byte entries take two pool slots
        elif tag == 15:
            at += 3
        elif tag == 16:
            at += 2
        elif tag == 17 or tag == 18 or tag == 19 or tag == 20:
            at += 4
        else:
            raise SystemExit(f"unknown constant tag {tag} at {at - 1}")
        index += 1
    return at


def utf8_name(data, cp_index):
    # Linear scan for the Utf8 entry (fine for these fixtures).
    count = u2(data, 8)
    at = 10
    index = 1
    while index < count:
        tag = data[at]
        at += 1
        if tag == 1:
            length = u2(data, at)
            if index == cp_index:
                return data[at + 2 : at + 2 + length]
            at += 2 + length
        elif tag == 7:
            at += 2
        elif tag in (9, 10, 11, 3, 4, 12):
            at += 4
        elif tag == 8:
            at += 2
        elif tag in (5, 6):
            at += 8
            index += 1  # 8-byte entries take two slots
        elif tag == 15:
            at += 3
        elif tag == 16:
            at += 2
        elif tag in (17, 18, 19, 20):
            at += 4
        else:
            raise SystemExit(f"unknown tag {tag}")
        index += 1
    raise SystemExit("constant not found")


def main():
    src, dst, method, row, field, value = sys.argv[1:7]
    row, value = int(row), int(value)
    data = bytearray(open(src, "rb").read())
    at = skip_constant_pool(data)
    at += 6  # access_flags, this_class, super_class
    interfaces = u2(data, at)
    at += 2 + 2 * interfaces
    # fields
    at += 2  # the fields_count itself
    for _ in range(u2(data, at - 2)):
        at += 6
        at = skip_attributes(data, at)
    # methods
    methods = u2(data, at)
    at += 2
    for _ in range(methods):
        name_index = u2(data, at + 2)
        if utf8_name(data, name_index) == method.encode():
            at += 6  # access, name, descriptor
            code_at = find_code_attribute(data, at)
            patch_exception_row(data, code_at, row, field, value)
            open(dst, "wb").write(bytes(data))
            return
        at += 6
        at = skip_attributes(data, at)
    raise SystemExit(f"method {method} not found")


def skip_attributes(data, at):
    at += 2  # the attributes_count itself
    for _ in range(u2(data, at - 2)):
        at += 6 + u4(data, at + 2)
    return at


def u4(data, at):
    return struct.unpack_from(">I", data, at)[0]


def find_code_attribute(data, at):
    count = u2(data, at)
    at += 2
    for _ in range(count):
        name_index = u2(data, at)
        length = u4(data, at + 2)
        if utf8_name(data, name_index) == b"Code":
            return at + 6  # past the attribute header
        at += 6 + length
    raise SystemExit("no Code attribute")


def patch_exception_row(data, code_at, row, field, value):
    max_stack, max_locals = u2(data, code_at), u2(data, code_at + 2)
    code_length = u4(data, code_at + 4)
    at = code_at + 8 + code_length
    table_length = u2(data, at)
    at += 2
    entry = at + row * 8
    offset = {"start": 0, "end": 2, "handler": 4}[field]
    old = u2(data, entry + offset)
    struct.pack_into(">H", data, entry + offset, value)
    print(f"row {row} {field}: {old} -> {value} (of {table_length} rows)")


main()
