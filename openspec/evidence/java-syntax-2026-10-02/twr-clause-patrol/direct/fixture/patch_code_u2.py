#!/usr/bin/env python3
"""Patch one 2-byte operand of one code instruction in one method of a Java 8 class file.

Reads the classfile format just far enough to find the named method's Code attribute
and rewrite a big-endian u16 at an absolute code offset (an `ldc` constant-pool
index, a branch delta, a local slot — whatever sits there).
Usage: patch_code_u2.py <in.class> <out.class> <method-name> <code-offset> <value>
"""
import struct
import sys


def u2(data, at):
    return struct.unpack_from(">H", data, at)[0]


def u4(data, at):
    return struct.unpack_from(">I", data, at)[0]


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
        elif tag in (9, 10, 11, 3, 4, 12):
            at += 4
        elif tag == 8:
            at += 2
        elif tag in (5, 6):
            at += 8
            index += 1  # 8-byte entries take two pool slots
        elif tag == 15:
            at += 3
        elif tag == 16:
            at += 2
        elif tag in (17, 18, 19, 20):
            at += 4
        else:
            raise SystemExit(f"unknown constant tag {tag} at {at - 1}")
        index += 1
    return at


def utf8_name(data, cp_index):
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
            index += 1
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


def skip_attributes(data, at):
    at += 2  # the attributes_count itself
    for _ in range(u2(data, at - 2)):
        at += 6 + u4(data, at + 2)
    return at


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


def main():
    src, dst, method, offset, value = sys.argv[1:6]
    width = int(sys.argv[6]) if len(sys.argv) > 6 else 2
    offset, value = int(offset), int(value)
    data = bytearray(open(src, "rb").read())
    at = skip_constant_pool(data)
    at += 6  # access_flags, this_class, super_class
    interfaces = u2(data, at)
    at += 2 + 2 * interfaces
    at += 2  # the fields_count itself
    for _ in range(u2(data, at - 2)):
        at += 6
        at = skip_attributes(data, at)
    methods = u2(data, at)
    at += 2
    for _ in range(methods):
        name_index = u2(data, at + 2)
        if utf8_name(data, name_index) == method.encode():
            at += 6  # access, name, descriptor
            code_at = find_code_attribute(data, at)
            code_length = u4(data, code_at + 4)
            if offset + width > code_length:
                raise SystemExit(f"offset {offset} past code length {code_length}")
            site = code_at + 8 + offset
            if width == 1:
                old = data[site]
                data[site] = value
            elif width == 2:
                old = u2(data, site)
                struct.pack_into(">H", data, site, value)
            else:
                raise SystemExit(f"unsupported width {width}")
            open(dst, "wb").write(bytes(data))
            print(f"{method} code+{offset}: u{width} {old} -> {value}")
            return
        at += 6
        at = skip_attributes(data, at)
    raise SystemExit(f"method {method} not found")


main()
