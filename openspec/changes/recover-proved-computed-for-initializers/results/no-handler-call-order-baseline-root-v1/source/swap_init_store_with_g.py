#!/usr/bin/env python3
"""Swap only `istore 6` and the immediately following `g()V` call in find()."""
import argparse
import hashlib
from pathlib import Path
import struct

OWNER = "ComputedForEffectControl"
METHOD = "find"
DESCRIPTOR = "([IIIIII)I"


def u1(data, at):
    return data[at]


def u2(data, at):
    return struct.unpack_from(">H", data, at)[0]


def u4(data, at):
    return struct.unpack_from(">I", data, at)[0]


def skip_attributes(data, at, count):
    for _ in range(count):
        length = u4(data, at + 2)
        at += 6 + length
    return at


def parse_class(data):
    if data[:4] != b"\xca\xfe\xba\xbe":
        raise ValueError("input is not a JVM class file")
    cp_count = u2(data, 8)
    cp = [None] * cp_count
    at = 10
    i = 1
    while i < cp_count:
        tag = u1(data, at)
        at += 1
        if tag == 1:
            length = u2(data, at)
            at += 2
            cp[i] = (tag, data[at:at + length].decode("utf-8"))
            at += length
        elif tag in (3, 4):
            cp[i] = (tag, data[at:at + 4])
            at += 4
        elif tag in (5, 6):
            cp[i] = (tag, data[at:at + 8])
            at += 8
            i += 1
        elif tag in (7, 8, 16, 19, 20):
            cp[i] = (tag, u2(data, at))
            at += 2
        elif tag in (9, 10, 11, 12, 17, 18):
            cp[i] = (tag, u2(data, at), u2(data, at + 2))
            at += 4
        elif tag == 15:
            cp[i] = (tag, u1(data, at), u2(data, at + 1))
            at += 3
        else:
            raise ValueError("unsupported constant-pool tag %d" % tag)
        i += 1

    def utf8(index):
        entry = cp[index]
        if entry is None or entry[0] != 1:
            raise ValueError("constant-pool entry %d is not UTF8" % index)
        return entry[1]

    def method_ref(index):
        entry = cp[index]
        if entry is None or entry[0] not in (10, 11):
            return None
        class_entry = cp[entry[1]]
        nat = cp[entry[2]]
        if class_entry[0] != 7 or nat[0] != 12:
            raise ValueError("malformed method reference")
        return utf8(class_entry[1]), utf8(nat[1]), utf8(nat[2])

    at += 6  # access_flags, this_class, super_class
    interfaces = u2(data, at)
    at += 2 + 2 * interfaces
    fields = u2(data, at)
    at += 2
    for _ in range(fields):
        attribute_count = u2(data, at + 6)
        at = skip_attributes(data, at + 8, attribute_count)
    methods_count = u2(data, at)
    at += 2
    target = None
    for _ in range(methods_count):
        name = utf8(u2(data, at + 2))
        descriptor = utf8(u2(data, at + 4))
        attribute_count = u2(data, at + 6)
        attr_at = at + 8
        for _ in range(attribute_count):
            attr_name = utf8(u2(data, attr_at))
            attr_length = u4(data, attr_at + 2)
            payload = attr_at + 6
            if name == METHOD and descriptor == DESCRIPTOR and attr_name == "Code":
                code_length = u4(data, payload + 4)
                code_start = payload + 8
                exception_count = u2(data, code_start + code_length)
                if exception_count != 0:
                    raise ValueError("find() must have no exception-table handlers")
                target = (code_start, code_length)
            attr_at += 6 + attr_length
        at = attr_at
    if target is None:
        raise ValueError("target find" + DESCRIPTOR + " Code attribute not found")
    return cp, method_ref, target


def instruction_length(code, pc):
    op = code[pc]
    if op in (0x10, 0x12, 0x15, 0x16, 0x17, 0x18, 0x19,
              0x36, 0x37, 0x38, 0x39, 0x3A, 0xA9, 0xBC):
        return 2
    if op in (0x11, 0x13, 0x14, 0x84,
              0x99, 0x9A, 0x9B, 0x9C, 0x9D, 0x9E,
              0x9F, 0xA0, 0xA1, 0xA2, 0xA3, 0xA4, 0xA5, 0xA6,
              0xA7, 0xA8, 0xB2, 0xB3, 0xB4, 0xB5, 0xB6, 0xB7,
              0xB8, 0xBB, 0xBD, 0xC0, 0xC1, 0xC6, 0xC7):
        return 3
    if op in (0xB9, 0xBA):
        return 5
    if op == 0xC5:
        return 4
    if op in (0xC8, 0xC9):
        return 5
    if op == 0xAA:
        base = pc + 1 + ((4 - ((pc + 1) & 3)) & 3)
        low, high = struct.unpack_from(">ii", code, base + 4)
        return base - pc + 12 + 4 * (high - low + 1)
    if op == 0xAB:
        base = pc + 1 + ((4 - ((pc + 1) & 3)) & 3)
        pairs = struct.unpack_from(">i", code, base + 4)[0]
        return base - pc + 8 + 8 * pairs
    if op == 0xC4:
        return 6 if code[pc + 1] == 0x84 else 4
    return 1


def decode(code):
    out = []
    pc = 0
    while pc < len(code):
        size = instruction_length(code, pc)
        if size <= 0 or pc + size > len(code):
            raise ValueError("invalid instruction encoding at BCI %d" % pc)
        out.append((pc, code[pc], bytes(code[pc:pc + size])))
        pc += size
    if pc != len(code):
        raise ValueError("instruction stream does not end at Code length")
    return out


def is_call(instruction, method_ref, name, descriptor):
    pc, opcode, raw = instruction
    if opcode != 0xB8:
        return False
    return method_ref(struct.unpack_from(">H", raw, 1)[0]) == (OWNER, name, descriptor)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    original = args.input.read_bytes()
    data = bytearray(original)
    _, method_ref, (code_start, code_length) = parse_class(data)
    code = bytearray(data[code_start:code_start + code_length])
    insns = decode(code)
    matches = []
    for i in range(len(insns) - 4):
        a, b, c, d, e = insns[i:i + 5]
        if (is_call(a, method_ref, "f", "()I")
                and b[1] == 0x04 and c[1] == 0x64
                and d[1] == 0x36 and d[2] == b"\x36\x06"
                and is_call(e, method_ref, "g", "()V")):
            matches.append((d, e))
    if len(matches) != 1:
        raise ValueError("expected exactly one f(); iconst_1; isub; istore 6; g() sequence; got %d" % len(matches))
    store, call_g = matches[0]
    start = store[0]
    end = call_g[0] + len(call_g[2])
    if end - start != len(call_g[2]) + len(store[2]):
        raise ValueError("swap range length mismatch")
    code[start:end] = call_g[2] + store[2]
    after = decode(code)
    if not any(
        is_call(after[i], method_ref, "f", "()I")
        and after[i + 1][1] == 0x04
        and after[i + 2][1] == 0x64
        and is_call(after[i + 3], method_ref, "g", "()V")
        and after[i + 4][1] == 0x36 and after[i + 4][2] == b"\x36\x06"
        for i in range(len(after) - 4)
    ):
        raise ValueError("post-patch sequence did not verify")
    data[code_start:code_start + code_length] = code
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_bytes(data)
    print("input_sha256=" + hashlib.sha256(original).hexdigest())
    print("output_sha256=" + hashlib.sha256(data).hexdigest())
    print("find_swap_bci=%d..%d" % (start, end))
    print("code_length=%d unchanged=true exception_table=empty" % code_length)


if __name__ == "__main__":
    main()
