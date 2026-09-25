#!/usr/bin/env python3
"""Compile the Java 8 subject and move update()'s handler start from BCI 0 to 7."""

from pathlib import Path
import struct
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parent
SOURCE = ROOT / "PostfixHandlerBoundary.java"
OUTPUT = ROOT / "v8" / "PostfixHandlerBoundary.class"


def u1(data, offset):
    return data[offset], offset + 1


def u2(data, offset):
    return struct.unpack_from(">H", data, offset)[0], offset + 2


def u4(data, offset):
    return struct.unpack_from(">I", data, offset)[0], offset + 4


def parse_and_patch(data):
    if data[:4] != b"\xca\xfe\xba\xbe":
        raise ValueError("javac output is not a class file")
    offset = 8
    cp_count, offset = u2(data, offset)
    utf8 = {}
    index = 1
    while index < cp_count:
        tag, offset = u1(data, offset)
        if tag == 1:
            length, offset = u2(data, offset)
            utf8[index] = data[offset:offset + length].decode("utf-8")
            offset += length
        elif tag in (3, 4):
            offset += 4
        elif tag in (5, 6):
            offset += 8
            index += 1
        elif tag in (7, 8, 16, 19, 20):
            offset += 2
        elif tag in (9, 10, 11, 12, 17, 18):
            offset += 4
        elif tag == 15:
            offset += 3
        else:
            raise ValueError(f"unsupported constant-pool tag {tag}")
        index += 1

    offset += 6  # access_flags, this_class, super_class
    interfaces_count, offset = u2(data, offset)
    offset += interfaces_count * 2

    def skip_attributes(pos, count):
        for _ in range(count):
            _, pos = u2(data, pos)
            length, pos = u4(data, pos)
            pos += length
        return pos

    fields_count, offset = u2(data, offset)
    for _ in range(fields_count):
        offset += 6
        attr_count, offset = u2(data, offset)
        offset = skip_attributes(offset, attr_count)

    methods_count, offset = u2(data, offset)
    patched = bytearray(data)
    matches = 0
    for _ in range(methods_count):
        _, offset = u2(data, offset)  # access_flags
        name_index, offset = u2(data, offset)
        _, offset = u2(data, offset)  # descriptor_index
        attr_count, offset = u2(data, offset)
        name = utf8.get(name_index)
        for _ in range(attr_count):
            attr_name_index, offset = u2(data, offset)
            attr_length, offset = u4(data, offset)
            attr_name = utf8.get(attr_name_index)
            attr_end = offset + attr_length
            if name == "update" and attr_name == "Code":
                code_length, code_offset = u4(data, offset + 4)
                code = data[code_offset:code_offset + code_length]
                expected_tail = bytes.fromhex("5c 2e 5b 04 60 4f ac 4b 02 ac")
                if len(code) != 16 or code[0] != 0xB8 or code[3] != 0xB8 or code[6:] != expected_tail:
                    raise ValueError(
                        f"update() bytecode differs from the expected javac shape: {code.hex()}"
                    )
                exceptions_count_offset = code_offset + code_length
                exceptions_count, table_offset = u2(data, exceptions_count_offset)
                if exceptions_count != 1:
                    raise ValueError(f"expected one update() handler, found {exceptions_count}")
                start_pc, _ = u2(data, table_offset)
                end_pc, _ = u2(data, table_offset + 2)
                handler_pc, _ = u2(data, table_offset + 4)
                if (start_pc, end_pc, handler_pc) != (0, 12, 13):
                    raise ValueError(
                        "unexpected javac exception range "
                        f"[{start_pc},{end_pc})->{handler_pc}"
                    )
                struct.pack_into(">H", patched, table_offset, 7)
                matches += 1
            offset = attr_end
    if matches != 1:
        raise ValueError(f"expected to patch exactly one update() Code attribute, got {matches}")
    return bytes(patched)


def main():
    with tempfile.TemporaryDirectory(prefix="postfix-handler-boundary-") as directory:
        build_dir = Path(directory)
        subprocess.run(
            ["javac", "-Xlint:-options", "--release", "8", "-g:none", "-d", str(build_dir), str(SOURCE)],
            check=True,
        )
        compiled = (build_dir / "PostfixHandlerBoundary.class").read_bytes()
    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT.write_bytes(parse_and_patch(compiled))
    print(f"wrote {OUTPUT.relative_to(ROOT)} ({OUTPUT.stat().st_size} bytes)")


if __name__ == "__main__":
    main()
