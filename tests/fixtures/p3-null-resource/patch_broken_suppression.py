#!/usr/bin/env python3
"""Make a verifier-valid same-length mutation of NullResourceCore.useNullResource."""

from __future__ import annotations

import hashlib
import pathlib
import sys


def u2(data: bytes, offset: int) -> int:
    return int.from_bytes(data[offset : offset + 2], "big")


def u4(data: bytes, offset: int) -> int:
    return int.from_bytes(data[offset : offset + 4], "big")


def skip_attributes(data: bytes, offset: int, count: int, pool: list[object]) -> int:
    for _ in range(count):
        name = pool[u2(data, offset)]
        length = u4(data, offset + 2)
        offset += 6 + length
    return offset


def method_code(data: bytes) -> tuple[int, bytes, str]:
    pool_count = u2(data, 8)
    pool: list[object] = [None] * pool_count
    offset = 10
    index = 1
    while index < pool_count:
        tag = data[offset]
        offset += 1
        if tag == 1:
            length = u2(data, offset)
            value = data[offset + 2 : offset + 2 + length].decode("utf-8")
            pool[index] = (tag, value)
            offset += 2 + length
        elif tag in (3, 4):
            pool[index] = (tag,)
            offset += 4
        elif tag in (5, 6):
            pool[index] = (tag,)
            offset += 8
            index += 1
        elif tag in (7, 8, 16, 19, 20):
            pool[index] = (tag, u2(data, offset))
            offset += 2
        elif tag in (9, 10, 11, 12, 17, 18):
            pool[index] = (tag, u2(data, offset), u2(data, offset + 2))
            offset += 4
        elif tag == 15:
            pool[index] = (tag, data[offset], u2(data, offset + 1))
            offset += 3
        else:
            raise ValueError(f"unsupported constant-pool tag {tag}")
        index += 1

    offset += 6  # access flags, this class, super class
    interfaces = u2(data, offset)
    offset += 2 + interfaces * 2
    fields = u2(data, offset)
    offset += 2
    for _ in range(fields):
        offset += 6
        count = u2(data, offset)
        offset = skip_attributes(data, offset + 2, count, pool)

    methods = u2(data, offset)
    offset += 2
    for _ in range(methods):
        name_index = u2(data, offset + 2)
        descriptor_index = u2(data, offset + 4)
        count = u2(data, offset + 6)
        method_name = pool[name_index][1]
        method_descriptor = pool[descriptor_index][1]
        attribute_offset = offset + 8
        for _ in range(count):
            attribute_name = pool[u2(data, attribute_offset)][1]
            length = u4(data, attribute_offset + 2)
            content = attribute_offset + 6
            if (method_name, method_descriptor, attribute_name) == (
                "useNullResource",
                "()V",
                "Code",
            ):
                code_length = u4(data, content + 4)
                code_offset = content + 8
                code = data[code_offset : code_offset + code_length]
                opcode = code[36]
                member_index = u2(code, 37)
                tag, class_index, name_type_index = pool[member_index]
                class_name = pool[pool[class_index][1]][1]
                _, name_index, descriptor_index = pool[name_type_index]
                called_name = pool[name_index][1]
                called_descriptor = pool[descriptor_index][1]
                target = f"{class_name}.{called_name}{called_descriptor}"
                if opcode != 0xB6 or target != (
                    "java/lang/Throwable.addSuppressed(Ljava/lang/Throwable;)V"
                ):
                    raise ValueError(
                        f"BCI 36 is {opcode:#x} {target}, expected the proved addSuppressed call"
                    )
                return code_offset, code, target
            attribute_offset += 6 + length
        offset = attribute_offset
    raise ValueError("useNullResource()V Code attribute not found")


def main() -> None:
    if len(sys.argv) != 3:
        raise SystemExit("usage: patch_broken_suppression.py INPUT.class OUTPUT.class")
    source = pathlib.Path(sys.argv[1]).read_bytes()
    code_offset, code, target = method_code(source)
    if code[36:39] != bytes((0xB6, code[37], code[38])):
        raise ValueError("unexpected instruction encoding at BCI 36")
    mutated = bytearray(source)
    mutated[code_offset + 36 : code_offset + 39] = b"\x58\x00\x00"
    output = bytes(mutated)
    pathlib.Path(sys.argv[2]).write_bytes(output)
    print(f"method=useNullResource()V Code.sha256={hashlib.sha256(code).hexdigest()}")
    print(f"replaced BCI 36 {target} with pop2; nop; nop")
    print(f"class.sha256={hashlib.sha256(output).hexdigest()} bytes={len(output)}")


if __name__ == "__main__":
    main()
