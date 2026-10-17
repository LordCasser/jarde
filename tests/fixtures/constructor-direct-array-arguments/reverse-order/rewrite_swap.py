#!/usr/bin/env python3
"""Insert exactly one SWAP before make's ([B[B)V invokespecial in a javac classfile."""
import argparse
import struct
from pathlib import Path


def u2(data, pos):
    return struct.unpack_from(">H", data, pos)[0]


def u4(data, pos):
    return struct.unpack_from(">I", data, pos)[0]


def p4(value):
    return struct.pack(">I", value)


def cp_table(data):
    count = u2(data, 8)
    cp = [None] * count
    pos = 10
    index = 1
    while index < count:
        tag = data[pos]
        start = pos
        pos += 1
        if tag == 1:
            size = u2(data, pos)
            pos += 2
            raw = data[pos:pos + size]
            pos += size
            # Names/descriptors in this control use ASCII modified UTF-8.
            cp[index] = (tag, raw.decode("utf-8"))
        elif tag in (3, 4):
            pos += 4
            cp[index] = (tag,)
        elif tag in (5, 6):
            pos += 8
            cp[index] = (tag,)
            index += 1
        elif tag in (7, 8, 16, 19, 20):
            cp[index] = (tag, u2(data, pos))
            pos += 2
        elif tag in (9, 10, 11, 12, 17, 18):
            cp[index] = (tag, u2(data, pos), u2(data, pos + 2))
            pos += 4
        elif tag == 15:
            cp[index] = (tag, data[pos], u2(data, pos + 1))
            pos += 3
        else:
            raise ValueError("unsupported constant-pool tag %d at index %d" % (tag, index))
        index += 1
    return cp, pos


def utf8(cp, index):
    item = cp[index]
    if item is None or item[0] != 1:
        raise ValueError("constant-pool entry %d is not Utf8" % index)
    return item[1]


def class_name(cp, index):
    item = cp[index]
    if item is None or item[0] != 7:
        raise ValueError("constant-pool entry %d is not Class" % index)
    return utf8(cp, item[1])


def memberref(cp, index):
    item = cp[index]
    if item is None or item[0] not in (10, 11):
        raise ValueError("invokespecial operand is not a method reference")
    owner = class_name(cp, item[1])
    nat = cp[item[2]]
    if nat is None or nat[0] != 12:
        raise ValueError("method reference has no NameAndType")
    return owner, utf8(cp, nat[1]), utf8(cp, nat[2])


def instruction_size(code, pc):
    op = code[pc]
    if op in (0x10, 0x12, 0x15, 0x16, 0x17, 0x18, 0x19,
              0x36, 0x37, 0x38, 0x39, 0x3a, 0xa9, 0xbc):
        return 2
    if op in (0x11, 0x13, 0x14, 0x84, 0x99, 0x9a, 0x9b, 0x9c,
              0x9d, 0x9e, 0x9f, 0xa0, 0xa1, 0xa2, 0xa3, 0xa4,
              0xa5, 0xa6, 0xa7, 0xa8, 0xb2, 0xb3, 0xb4, 0xb5,
              0xb6, 0xb7, 0xb8, 0xbb, 0xbd, 0xc0, 0xc1, 0xc6,
              0xc7):
        return 3
    if op in (0xb9, 0xba, 0xc8, 0xc9):
        return 5
    if op == 0xc5:
        return 4
    if op == 0xaa:
        pad = (4 - ((pc + 1) & 3)) & 3
        base = pc + 1 + pad
        low = struct.unpack_from(">i", code, base + 4)[0]
        high = struct.unpack_from(">i", code, base + 8)[0]
        return 1 + pad + 12 + 4 * (high - low + 1)
    if op == 0xab:
        pad = (4 - ((pc + 1) & 3)) & 3
        base = pc + 1 + pad
        pairs = struct.unpack_from(">i", code, base + 4)[0]
        return 1 + pad + 8 + 8 * pairs
    if op == 0xc4:
        return 6 if code[pc + 1] == 0x84 else 4
    return 1


def parse_code(data, cp, start, length):
    end = start + length
    if end > len(data) or length < 12:
        raise ValueError("truncated Code attribute")
    code_len = u4(data, start + 4)
    code_start = start + 8
    code_end = code_start + code_len
    if code_end + 4 > end:
        raise ValueError("truncated Code body")
    exception_count = u2(data, code_end)
    if exception_count != 0:
        raise ValueError("make has an exception table; refusing rewrite")
    attrs_count_pos = code_end + 2
    attr_count = u2(data, attrs_count_pos)
    if attr_count != 0:
        names = []
        pos = attrs_count_pos + 2
        for _ in range(attr_count):
            name_index = u2(data, pos)
            size = u4(data, pos + 2)
            names.append(utf8(cp, name_index))
            pos += 6 + size
        raise ValueError("make has Code subattributes %r; refusing rewrite" % names)
    if attrs_count_pos + 2 != end:
        raise ValueError("Code attribute length does not match its fields")
    return code_start, code_len


def locate_make(data, cp, pos):
    # access_flags, this_class, super_class, interfaces
    this_class = u2(data, pos + 2)
    owner = class_name(cp, this_class)
    pos += 6
    interfaces = u2(data, pos)
    pos += 2 + 2 * interfaces
    fields = u2(data, pos)
    pos += 2
    for _ in range(fields):
        attr_count = u2(data, pos + 6)
        pos += 8
        for _ in range(attr_count):
            size = u4(data, pos + 2)
            pos += 6 + size
    method_count = u2(data, pos)
    pos += 2
    matches = []
    for _ in range(method_count):
        access = u2(data, pos)
        name = utf8(cp, u2(data, pos + 2))
        desc = utf8(cp, u2(data, pos + 4))
        attrs = u2(data, pos + 6)
        pos += 8
        for _ in range(attrs):
            name_index = u2(data, pos)
            size = u4(data, pos + 2)
            attr_name = utf8(cp, name_index)
            attr_start = pos + 6
            if name == "make" and desc == "()LReverseArrayOrder;" and attr_name == "Code":
                matches.append((owner, access, attr_start, size))
            pos += 6 + size
    if len(matches) != 1:
        raise ValueError("expected exactly one make()LReverseArrayOrder; Code attribute")
    return matches[0]


def rewrite(data):
    if u4(data, 0) != 0xCAFEBABE:
        raise ValueError("input is not a classfile")
    cp, cp_end = cp_table(data)
    owner, access, attr_start, attr_len = locate_make(data, cp, cp_end)
    if access & 0x0008 == 0:
        raise ValueError("make must be static")
    code_start, code_len = parse_code(data, cp, attr_start, attr_len)
    code = data[code_start:code_start + code_len]
    forbidden = set(range(0x99, 0xa9)) | {0xaa, 0xab, 0xc6, 0xc7, 0xc8, 0xc9}
    invoke_offsets = []
    pc = 0
    while pc < len(code):
        op = code[pc]
        if op in forbidden:
            raise ValueError("make contains control flow at BCI %d; refusing rewrite" % pc)
        size = instruction_size(code, pc)
        if size <= 0 or pc + size > len(code):
            raise ValueError("invalid instruction length at BCI %d" % pc)
        if op == 0xb7:
            cp_index = struct.unpack_from(">H", code, pc + 1)[0]
            if memberref(cp, cp_index) == (owner, "<init>", "([B[B)V"):
                invoke_offsets.append(pc)
        pc += size
    if pc != len(code) or len(invoke_offsets) != 1:
        raise ValueError("expected one complete constructor call, found %r" % invoke_offsets)
    invoke = invoke_offsets[0]

    # Whitelist this fixture's straight-line javac shape so a class from a different compiler
    # layout cannot be silently rewritten by this small fixture tool.
    allowed = set([0x57, 0x59, 0x5f, 0x10, 0x11, 0x12, 0x13, 0x14,
                   0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x10,
                   0xb7, 0xb8, 0xbb, 0xbc, 0xbd, 0xb0])
    pc = 0
    while pc < len(code):
        op = code[pc]
        if op not in allowed:
            raise ValueError("unreviewed opcode 0x%02x at BCI %d" % (op, pc))
        pc += instruction_size(code, pc)

    updated = bytearray(data[:code_start + invoke] + b"\x5f" + data[code_start + invoke:])
    struct.pack_into(">I", updated, attr_start - 4, attr_len + 1)
    struct.pack_into(">I", updated, attr_start + 4, code_len + 1)
    return bytes(updated)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("input", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    original = args.input.read_bytes()
    rewritten = rewrite(original)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_bytes(rewritten)


if __name__ == "__main__":
    main()
