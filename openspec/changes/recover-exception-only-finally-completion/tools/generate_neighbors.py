#!/usr/bin/env python3
"""Make valid class-file edits around sealed FinallyCompletion.tryThrowsFinallyRuns().

Use only on a copied canonical 907-byte input; the default source is the repository's sealed
javac8 fixture. The output is private test material. A Rust guard test must first assert each
variant's bytecode/IR is Complete, then assert the exception-only proof refuses it. Root owns
strict JVM verification and must keep any class that fails that gate out of proof-negative claims.
"""
from __future__ import annotations

import argparse
import struct
from pathlib import Path


def u2(data: bytes | bytearray, pos: int) -> int:
    return struct.unpack_from(">H", data, pos)[0]


def u4(data: bytes | bytearray, pos: int) -> int:
    return struct.unpack_from(">I", data, pos)[0]


def p2(data: bytearray, pos: int, value: int) -> None:
    struct.pack_into(">H", data, pos, value)


def p4(data: bytearray, pos: int, value: int) -> None:
    struct.pack_into(">I", data, pos, value)


def parse_pool(data: bytes | bytearray):
    count = u2(data, 8)
    cp: list[tuple[int, object] | None] = [None] * count
    pos = 10
    index = 1
    while index < count:
        tag = data[pos]
        pos += 1
        if tag == 1:
            length = u2(data, pos)
            pos += 2
            cp[index] = (tag, bytes(data[pos : pos + length]).decode("utf-8"))
            pos += length
        elif tag in (3, 4):
            pos += 4
        elif tag in (5, 6):
            pos += 8
            index += 1
        elif tag in (7, 8, 16, 19, 20):
            cp[index] = (tag, u2(data, pos))
            pos += 2
        elif tag in (9, 10, 11, 12, 17, 18):
            pos += 4
        elif tag == 15:
            pos += 3
        else:
            raise ValueError(f"unknown constant-pool tag {tag} at #{index}")
        index += 1
    return cp, pos


def utf(cp, index: int) -> str:
    value = cp[index]
    if value is None or value[0] != 1:
        raise ValueError(f"constant #{index} is not UTF8")
    return str(value[1])


def class_named(cp, internal_name: str) -> int:
    for index, value in enumerate(cp):
        if value and value[0] == 7 and utf(cp, int(value[1])) == internal_name:
            return index
    raise ValueError(f"constant pool has no CONSTANT_Class {internal_name}")


def target_code(data: bytes | bytearray):
    """Parse the complete classfile and return the target Code offsets/layout."""
    cp, pos = parse_pool(data)
    if pos + 8 > len(data):
        raise ValueError("truncated class header")
    pos += 6  # access_flags, this_class, super_class
    interfaces = u2(data, pos)
    pos += 2 + interfaces * 2

    def skip_members(cursor: int, count: int) -> int:
        for _ in range(count):
            if cursor + 8 > len(data):
                raise ValueError("truncated member header")
            attr_count = u2(data, cursor + 6)
            cursor += 8
            for _ in range(attr_count):
                if cursor + 6 > len(data):
                    raise ValueError("truncated member attribute header")
                length = u4(data, cursor + 2)
                cursor += 6 + length
                if cursor > len(data):
                    raise ValueError("member attribute runs past classfile")
        return cursor

    fields = u2(data, pos)
    pos = skip_members(pos + 2, fields)
    if pos + 2 > len(data):
        raise ValueError("truncated methods_count")
    methods = u2(data, pos)
    pos += 2
    found = None
    for _ in range(methods):
        if pos + 8 > len(data):
            raise ValueError("truncated method header")
        name_index = u2(data, pos + 2)
        desc_index = u2(data, pos + 4)
        attr_count = u2(data, pos + 6)
        is_target = utf(cp, name_index) == "tryThrowsFinallyRuns" and utf(cp, desc_index) == "()I"
        pos += 8
        for _ in range(attr_count):
            if pos + 6 > len(data):
                raise ValueError("truncated method attribute header")
            attr_name = utf(cp, u2(data, pos))
            attr_length = u4(data, pos + 2)
            info = pos + 6
            attr_end = info + attr_length
            if attr_end > len(data):
                raise ValueError("method attribute runs past classfile")
            if is_target and attr_name == "Code":
                code_length = u4(data, info + 4)
                code_start = info + 8
                if code_start + code_length + 2 > attr_end:
                    raise ValueError("Code array runs past its Code attribute")
                exception_count_pos = code_start + code_length
                exception_count = u2(data, exception_count_pos)
                exception_start = exception_count_pos + 2
                exception_end = exception_start + exception_count * 8
                if exception_end + 2 > attr_end:
                    raise ValueError("exception table/nested-attribute count runs past Code")
                nested_count = u2(data, exception_end)
                nested_pos = exception_end + 2
                for _ in range(nested_count):
                    if nested_pos + 6 > attr_end:
                        raise ValueError("truncated nested Code attribute")
                    nested_pos += 6 + u4(data, nested_pos + 2)
                if nested_pos != attr_end:
                    raise ValueError("Code attribute length does not match its nested attributes")
                if found is not None:
                    raise ValueError("target has duplicate Code attributes")
                if code_length != 19 or exception_count < 1:
                    raise ValueError(
                        f"expected 19-byte target with at least one row, got {code_length}/{exception_count}"
                    )
                found = {
                    "cp": cp,
                    "code_start": code_start,
                    "code_length": code_length,
                    "exception_count_pos": exception_count_pos,
                    "exception_start": exception_start,
                    "code_attr_length_pos": pos + 2,
                    "code_attr_length": attr_length,
                    "throwable_class": class_named(cp, "java/lang/Throwable"),
                }
            pos = attr_end
    if found is None:
        raise ValueError("target method Code attribute not found")
    if pos + 2 > len(data):
        raise ValueError("truncated class attributes_count")
    class_attributes = u2(data, pos)
    pos += 2
    for _ in range(class_attributes):
        if pos + 6 > len(data):
            raise ValueError("truncated class attribute header")
        pos += 6 + u4(data, pos + 2)
        if pos > len(data):
            raise ValueError("class attribute runs past classfile")
    if pos != len(data):
        raise ValueError(f"trailing bytes after class attributes: {len(data) - pos}")
    return found


def instruction_boundaries(code: bytes) -> tuple[set[int], list[tuple[int, int]]]:
    """Decode target opcodes structurally; this is not a JVM type verifier."""
    starts: set[int] = set()
    branches: list[tuple[int, int]] = []
    pc = 0
    while pc < len(code):
        starts.add(pc)
        op = code[pc]
        if op in (0xAA, 0xAB):  # tableswitch / lookupswitch
            pad = (4 - ((pc + 1) & 3)) & 3
            base = pc + 1 + pad
            if base + 8 > len(code):
                raise ValueError("truncated switch header")
            default = struct.unpack_from(">i", code, base)[0]
            branches.append((pc, pc + default))
            if op == 0xAA:
                low, high = struct.unpack_from(">ii", code, base + 4)
                if high < low:
                    raise ValueError("tableswitch high is below low")
                count = high - low + 1
                end = base + 12 + count * 4
                if end > len(code):
                    raise ValueError("truncated tableswitch targets")
                for i in range(count):
                    offset = struct.unpack_from(">i", code, base + 12 + i * 4)[0]
                    branches.append((pc, pc + offset))
            else:
                pairs = struct.unpack_from(">i", code, base + 4)[0]
                if pairs < 0:
                    raise ValueError("negative lookupswitch pair count")
                end = base + 8 + pairs * 8
                if end > len(code):
                    raise ValueError("truncated lookupswitch pairs")
                for i in range(pairs):
                    offset = struct.unpack_from(">i", code, base + 12 + i * 8)[0]
                    branches.append((pc, pc + offset))
            pc = end
            continue
        if op == 0xC4:  # wide
            if pc + 2 > len(code):
                raise ValueError("truncated wide opcode")
            pc += 6 if code[pc + 1] == 0x84 else 4
            continue
        width = 1
        if op in (0x10, 0x12, 0x15, 0x16, 0x17, 0x18, 0x19,
                  0x36, 0x37, 0x38, 0x39, 0x3A, 0xA9, 0xBC):
            width = 2
        elif op in (0x11, 0x13, 0x14, 0x84,
                    0x99, 0x9A, 0x9B, 0x9C, 0x9D, 0x9E,
                    0x9F, 0xA0, 0xA1, 0xA2, 0xA3, 0xA4, 0xA5, 0xA6,
                    0xA7, 0xA8, 0xB2, 0xB3, 0xB4, 0xB5, 0xB6, 0xB7,
                    0xB8, 0xBB, 0xBD, 0xC0, 0xC1, 0xC6, 0xC7):
            width = 3
            if 0x99 <= op <= 0xA8 or op in (0xC6, 0xC7):
                offset = struct.unpack_from(">h", code, pc + 1)[0]
                branches.append((pc, pc + offset))
        elif op in (0xB9, 0xBA, 0xC8, 0xC9):
            width = 5 if op in (0xB9, 0xBA) else 5
            if op in (0xC8, 0xC9):
                offset = struct.unpack_from(">i", code, pc + 1)[0]
                branches.append((pc, pc + offset))
        elif op == 0xC5:
            width = 4
        if pc + width > len(code):
            raise ValueError(f"truncated opcode 0x{op:02x} at BCI {pc}")
        pc += width
    if pc != len(code):
        raise ValueError("instruction stream does not end at code_length")
    return starts, branches


def validate_structure(data: bytes, label: str) -> None:
    layout = target_code(data)  # also validates all class/member/attribute extents and EOF
    code = data[layout["code_start"] : layout["code_start"] + layout["code_length"]]
    starts, branches = instruction_boundaries(code)
    expected_starts = {0, 2, 5, 6, 9, 10, 11, 13, 16, 17, 18}
    if starts != expected_starts:
        raise ValueError(f"{label}: target instruction boundaries changed: {sorted(starts)}")
    for origin, target in branches:
        if target not in starts:
            raise ValueError(f"{label}: branch at {origin} targets non-instruction BCI {target}")
    exception_count = u2(data, layout["exception_count_pos"])
    for index in range(exception_count):
        start, end, handler, _catch = struct.unpack_from(">HHHH", data, layout["exception_start"] + index * 8)
        if start not in starts or end > len(code) or (end != len(code) and end not in starts):
            raise ValueError(f"{label}: exception range is not instruction-aligned: {(start, end)}")
        if handler not in starts:
            raise ValueError(f"{label}: handler is not instruction-aligned: {handler}")

def variants(original: bytes) -> dict[str, bytes]:
    layout = target_code(original)
    out: dict[str, bytes] = {}

    # Catch-all row ends exactly at the explicit athrow BCI 9. The instruction stream remains
    # complete, but that final throw is no longer protected by the catch-all.
    changed = bytearray(original)
    p2(changed, layout["exception_start"] + 2, 9)
    out["row-shrunk.class"] = bytes(changed)

    # Positive metamorphic boundary: move only the nonthrowing handler binding store at BCI 10
    # outside the row. Every body throw site at BCIs 2, 6 and 9 remains covered.
    changed = bytearray(original)
    p2(changed, layout["exception_start"] + 2, 10)
    out["binding-unprotected.class"] = bytes(changed)

    # The row now also covers the cleanup invocation at BCI 13, creating a real exceptional edge
    # back into its own handler. This is a valid class-file table shape, not a truncated parse.
    changed = bytearray(original)
    p2(changed, layout["exception_start"] + 2, 16)
    out["cleanup-protected.class"] = bytes(changed)

    # A typed Throwable entry competes with the catch-all over the same protected range.
    # Put it first so its precedence is observable; both entries enter the existing handler.
    changed = bytearray(original)
    count_pos = layout["exception_count_pos"]
    row_pos = layout["exception_start"]
    throwable = layout["throwable_class"]
    changed[row_pos:row_pos] = struct.pack(">HHHH", 0, 11, 10, throwable)
    p2(changed, count_pos, 2)
    p4(changed, layout["code_attr_length_pos"], layout["code_attr_length"] + 8)
    out["competing-row.class"] = bytes(changed)

    # BCI 17 is the one-byte aload_0 that forwards the saved caught exception to BCI 18 athrow.
    # Null remains verifier-assignable as a reference but is not the caught SSA value.
    changed = bytearray(original)
    changed[layout["code_start"] + 17] = 0x01  # aconst_null
    out["wrong-rethrow-value.class"] = bytes(changed)

    # Replace the body throw at BCIs 6..9 with a normal branch carrying null to the handler's
    # Throwable stack entry. It creates an extra normal entrance into the exceptional cleanup.
    changed = bytearray(original)
    changed[layout["code_start"] + 6 : layout["code_start"] + 10] = bytes(
        (0x01, 0xA7, 0x00, 0x03)  # aconst_null; goto +3 (from BCI 7 to handler BCI 10)
    )
    out["extra-normal-handler-entry.class"] = bytes(changed)

    # Turn the handler tail into an int return: an explicit nonlocal completion in the finally
    # block suppresses the pending primary. The method descriptor is ()I, so this is verifier-fit.
    changed = bytearray(original)
    changed[layout["code_start"] + 17 : layout["code_start"] + 19] = bytes((0x03, 0xAC))
    out["finally-explicit-return.class"] = bytes(changed)

    # Excluding only the nonthrowing binding preserves every protected throw-site.
    changed = bytearray(original)
    p2(changed, layout["exception_start"] + 2, 10)
    out["binding-unprotected.class"] = bytes(changed)

    return out


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--input",
        type=Path,
        default=Path("tests/fixtures/recover-exception-only-finally-completion/sealed/javac8/FinallyCompletion.class"),
    )
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    original = args.input.read_bytes()
    args.output.mkdir(parents=True, exist_ok=True)
    for name, data in variants(original).items():
        validate_structure(data, name)
        (args.output / name).write_bytes(data)
        print(f"{name}\t{len(data)} bytes (structurally bounded; JVM verifier gate remains with root)")


if __name__ == "__main__":
    main()
