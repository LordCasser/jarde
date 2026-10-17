#!/usr/bin/env python3
"""Emit a minimal v49 class with two jsr calls to a loop containing lookupswitch."""
from pathlib import Path
import argparse
import struct


def u1(value):
    return struct.pack(">B", value)


def u2(value):
    return struct.pack(">H", value)


def s2(value):
    return struct.pack(">h", value)


def u4(value):
    return struct.pack(">I", value)


def utf8(value):
    data = value.encode("ascii")
    return u1(1) + u2(len(data)) + data


def patch_s2(code, operand_bci, opcode_bci, target_bci):
    struct.pack_into(">h", code, operand_bci, target_bci - opcode_bci)


def patch_s4(code, operand_bci, opcode_bci, target_bci):
    struct.pack_into(">i", code, operand_bci, target_bci - opcode_bci)


def build_code():
    code = bytearray()
    code += bytes((0x03, 0x3D, 0x03, 0x3E))  # iconst_0; istore_2; iconst_0; istore_3

    first_jsr = len(code)
    code += bytes((0xA8, 0x00, 0x00))
    second_jsr = len(code)
    code += bytes((0xA8, 0x00, 0x00))
    code += bytes((0x1D, 0xAC))  # iload_3; ireturn

    subroutine = len(code)
    code += bytes((0x4C, 0x03, 0x3D))  # astore_1; iconst_0; istore_2
    loop_header = len(code)
    code += bytes((0x1C, 0x1A))  # iload_2; iload_0
    loop_exit_branch = len(code)
    code += bytes((0xA2, 0x00, 0x00))  # if_icmpge loop_exit
    code += u1(0x1C)  # iload_2, selector for lookupswitch
    switch_bci = len(code)
    code += u1(0xAB)
    while len(code) & 3:
        code += u1(0)
    default_offset = len(code)
    code += u4(0)
    code += struct.pack(">i", 2)
    code += struct.pack(">i", 0)
    case_zero_offset = len(code)
    code += u4(0)
    code += struct.pack(">i", 2)
    case_two_offset = len(code)
    code += u4(0)

    case_zero = len(code)
    code += bytes((0x84, 0x03, 0x01))  # iinc 3, 1
    goto_join_zero = len(code)
    code += bytes((0xA7, 0x00, 0x00))
    case_two = len(code)
    code += bytes((0x84, 0x03, 0x02))  # iinc 3, 2
    goto_join_two = len(code)
    code += bytes((0xA7, 0x00, 0x00))
    default_case = len(code)
    code += bytes((0x84, 0x03, 0x03))  # iinc 3, 3
    join = len(code)
    code += bytes((0x84, 0x02, 0x01))  # iinc 2, 1
    goto_header = len(code)
    code += bytes((0xA7, 0x00, 0x00))
    loop_exit = len(code)
    code += bytes((0xA9, 0x01))  # ret 1

    patch_s2(code, first_jsr + 1, first_jsr, subroutine)
    patch_s2(code, second_jsr + 1, second_jsr, subroutine)
    patch_s2(code, loop_exit_branch + 1, loop_exit_branch, loop_exit)
    patch_s2(code, goto_join_zero + 1, goto_join_zero, join)
    patch_s2(code, goto_join_two + 1, goto_join_two, join)
    patch_s2(code, goto_header + 1, goto_header, loop_header)
    patch_s4(code, default_offset, switch_bci, default_case)
    patch_s4(code, case_zero_offset, switch_bci, case_zero)
    patch_s4(code, case_two_offset, switch_bci, case_two)

    assert subroutine == 12
    assert switch_bci % 4 == 1
    assert loop_exit == 69
    assert len(code) == 71
    return bytes(code)


def build_class():
    code = build_code()
    # Constant pool: class name, this/super classes, method name/descriptor, Code.
    cp = b"".join(
        (
            utf8("SwitchLoopJsrClone"),       # 1
            u1(7) + u2(1),                    # 2 Class
            utf8("java/lang/Object"),         # 3
            u1(7) + u2(3),                    # 4 Class
            utf8("run"),                      # 5
            utf8("(I)I"),                     # 6
            utf8("Code"),                    # 7
        )
    )
    cp_count = 8
    code_info = u2(2) + u2(4) + u4(len(code)) + code + u2(0) + u2(0)
    method = (
        u2(0x0009)  # public static
        + u2(5)
        + u2(6)
        + u2(1)
        + u2(7)
        + u4(len(code_info))
        + code_info
    )
    return (
        u4(0xCAFEBABE)
        + u2(0)
        + u2(49)  # Java 5-era classfile, so no StackMapTable is required.
        + u2(cp_count)
        + cp
        + u2(0x0021)  # public, super
        + u2(2)
        + u2(4)
        + u2(0)  # interfaces
        + u2(0)  # fields
        + u2(1)
        + method
        + u2(0)  # class attributes
    )


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("output_class", type=Path)
    args = parser.parse_args()
    args.output_class.parent.mkdir(parents=True, exist_ok=True)
    args.output_class.write_bytes(build_class())


if __name__ == "__main__":
    main()
