#!/usr/bin/env python3
"""Emit a major-49 class with one effectful loop header cloned at two jsr call sites."""
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
    # run(I)I has two jsr call sites at BCIs 2 and 5, both targeting subroutine BCI 10.
    # The subroutine stores the return address in local 0, initializes local 2, then reaches
    # the same physical loop header at BCI 23. Normalization should give the header two distinct
    # clone paths. The loop body increments local 2 and returns to the header; ret 0 returns to
    # the corresponding call site. tick(I)I increments static calls, so the header call is
    # observably effectful. No method is invoked by the verification driver.
    run = bytes.fromhex(
        "1a3ca80008a800051bac4b033d"
        "00000000000000000000"
        "1cb8000b3d1c1ba20009840201a7fff3a900"
    )
    assert len(run) == 41
    tick = bytes.fromhex("b2000f0460b3000f1aac")
    assert len(tick) == 10

    cp = b"".join(
        (
            utf8("StatementfulJsrCloneHeader"), # 1
            u1(7) + u2(1),                      # 2 Class
            utf8("java/lang/Object"),           # 3
            u1(7) + u2(3),                      # 4 Class
            utf8("run"),                        # 5
            utf8("(I)I"),                       # 6
            utf8("Code"),                      # 7
            utf8("tick"),                       # 8
            utf8("(I)I"),                       # 9
            u1(12) + u2(8) + u2(9),             # 10 NameAndType
            u1(10) + u2(2) + u2(10),            # 11 Methodref
            utf8("calls"),                      # 12
            utf8("I"),                          # 13
            u1(12) + u2(12) + u2(13),           # 14 NameAndType
            u1(9) + u2(2) + u2(14),             # 15 Fieldref
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
