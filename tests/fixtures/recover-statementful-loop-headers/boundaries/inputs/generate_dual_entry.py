#!/usr/bin/env python3
"""Emit a small major-49 class whose effectful header loop has a second body entry."""
from pathlib import Path
import argparse
import struct


def u1(value):
    return struct.pack(">B", value)


def u2(value):
    return struct.pack(">H", value)


def u4(value):
    return struct.pack(">I", value)


def utf8(value):
    data = value.encode("ascii")
    return u1(1) + u2(len(data)) + data


def code_attribute(code, max_stack, max_locals):
    info = u2(max_stack) + u2(max_locals) + u4(len(code)) + code + u2(0) + u2(0)
    return u2(7) + u4(len(info)) + info


def method(access, name_index, descriptor_index, code, max_stack, max_locals):
    return (
        u2(access)
        + u2(name_index)
        + u2(descriptor_index)
        + u2(1)
        + code_attribute(code, max_stack, max_locals)
    )


def build_class():
    # run(II)I:
    #   0 iconst_0; 1 istore_2; 2 iload_1; 3 ifne 19; 6 goto 9
    #   9 iload_2; 10 invokestatic tick; 13 istore_2; 14 iload_2; 15 iload_0
    #   16 if_icmpge 25; 19 iinc 2,1; 22 goto 9; 25 iload_2; 26 ireturn
    # The preheader's conditional edge at 3 enters body BCI 19 directly. The header's
    # fallthrough also reaches BCI 19, and body goto 22 returns to header BCI 9.
    run = bytes.fromhex("033d1b9a0010a700031cb8000b3d1c1aa20009840201a7fff31cac")
    assert len(run) == 27

    # tick(I)I updates static calls, then returns its input. The header invocation is
    # therefore observable and cannot be treated as a pure comparison operand.
    tick = bytes.fromhex("b2000f0460b3000f1aac")
    assert len(tick) == 10

    cp = b"".join(
        (
            utf8("StatementfulDualEntry"),       # 1
            u1(7) + u2(1),                        # 2 Class
            utf8("java/lang/Object"),             # 3
            u1(7) + u2(3),                        # 4 Class
            utf8("run"),                          # 5
            utf8("(II)I"),                        # 6
            utf8("Code"),                         # 7
            utf8("tick"),                         # 8
            utf8("(I)I"),                         # 9
            u1(12) + u2(8) + u2(9),               # 10 NameAndType
            u1(10) + u2(2) + u2(10),             # 11 Methodref
            utf8("calls"),                        # 12
            utf8("I"),                            # 13
            u1(12) + u2(12) + u2(13),             # 14 NameAndType
            u1(9) + u2(2) + u2(14),               # 15 Fieldref
        )
    )
    fields = u2(0x0009) + u2(12) + u2(13) + u2(0)  # public static calls:I
    methods = method(0x0009, 5, 6, run, 2, 3) + method(0x000a, 8, 9, tick, 2, 1)
    return (
        u4(0xCAFEBABE)
        + u2(0)
        + u2(49)
        + u2(16)
        + cp
        + u2(0x0021)
        + u2(2)
        + u2(4)
        + u2(0)
        + u2(1)
        + fields
        + u2(2)
        + methods
        + u2(0)
    )


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("output_class", type=Path)
    args = parser.parse_args()
    args.output_class.parent.mkdir(parents=True, exist_ok=True)
    args.output_class.write_bytes(build_class())


if __name__ == "__main__":
    main()
