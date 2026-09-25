#!/usr/bin/env python3
"""Freeze a Java 8 class whose post-range call precedes the catch handler in one block."""

from pathlib import Path
import struct
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parent
SOURCE = ROOT / "BoundaryPostfixProbe.java"
MONITOR_SOURCE = ROOT / "BoundaryMonitorProbe.java"
OUTPUT = ROOT / "v8" / "BoundaryPostfixProbe.class"
MONITOR_OUTPUT = ROOT / "v8" / "BoundaryMonitorProbe.class"


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
        _, offset = u2(data, offset)
        name_index, offset = u2(data, offset)
        _, offset = u2(data, offset)
        attr_count, offset = u2(data, offset)
        name = utf8.get(name_index)
        for _ in range(attr_count):
            attr_name_index, offset = u2(data, offset)
            attr_length, offset = u4(data, offset)
            attr_name = utf8.get(attr_name_index)
            attr_end = offset + attr_length
            if name == "choosePostfix" and attr_name == "Code":
                code_length, code_offset = u4(data, offset + 4)
                code = data[code_offset:code_offset + code_length]
                if (
                    len(code) != 15
                    or code[2:6] != bytes.fromhex("4b a7 00 07")
                    or code[6] != 0x4c
                    or code[9] != 0xb0
                    or code[10] != 0x2a
                    or code[11] != 0xb8
                    or code[14] != 0xb0
                ):
                    raise ValueError(f"unexpected choosePostfix bytecode: {code.hex()}")
                # Normal fallthrough now evaluates `after` after the range and returns. The catch
                # handler follows that terminal return, so it cannot be reached by normal flow.
                new_code = code[0:3] + code[10:15] + code[6:10]
                patched[code_offset:code_offset + len(new_code)] = new_code
                length_offset = offset + 4
                struct.pack_into(">I", patched, length_offset, len(new_code))

                table_offset = code_offset + code_length
                exception_count, table_offset = u2(data, table_offset)
                if exception_count != 1:
                    raise ValueError(f"expected one catch row, found {exception_count}")
                start_pc, end_pc, handler_pc, catch_type = struct.unpack_from(">HHHH", data, table_offset)
                if (start_pc, end_pc, handler_pc) != (0, 3, 6):
                    raise ValueError(
                        f"unexpected javac range [{start_pc},{end_pc})->{handler_pc}"
                    )
                struct.pack_into(">HHHH", patched, table_offset, 0, 3, 8, catch_type)
                nested_count_offset = table_offset + 8
                nested_count, nested_offset = u2(data, nested_count_offset)
                if nested_count != 1:
                    raise ValueError(f"expected one Code subattribute, found {nested_count}")
                attr_index, nested_offset = u2(data, nested_offset)
                nested_length, nested_offset = u4(data, nested_offset)
                if utf8.get(attr_index) != "StackMapTable":
                    raise ValueError("expected StackMapTable as the only Code subattribute")
                stackmap = data[nested_offset:nested_offset + nested_length]
                # javac emitted a handler frame at BCI 6 and a normal-join frame at BCI 10. The
                # latter is no longer a control-flow target; move the sole handler frame to BCI 8.
                if len(stackmap) != 12 or stackmap[0:2] != b"\x00\x02" or stackmap[2] != 70:
                    raise ValueError(f"unexpected StackMapTable: {stackmap.hex()}")
                first_frame = bytes([72]) + stackmap[3:6]
                new_stackmap = b"\x00\x01" + first_frame
                patched[nested_offset:nested_offset + nested_length] = new_stackmap
                struct.pack_into(">I", patched, nested_offset - 4, len(new_stackmap))
                # The Code attribute loses three code bytes and six stack-map bytes.
                struct.pack_into(">I", patched, offset - 4, attr_length - 9)
                del patched[code_offset + len(new_code):code_offset + code_length]
                matches += 1
            offset = attr_end
    if matches != 1:
        raise ValueError(f"expected one choosePostfix Code attribute, got {matches}")

    # The Code and StackMapTable lengths were reduced above; the byte-array deletions shifted the
    # later class attributes without changing their contents.
    return bytes(patched)


def main():
    with tempfile.TemporaryDirectory(prefix="typed-catch-boundary-") as directory:
        build_dir = Path(directory)
        subprocess.run(
            [
                "javac",
                "--release",
                "8",
                "-g:none",
                "-Xlint:-options",
                "-d",
                str(build_dir),
                str(SOURCE),
                str(MONITOR_SOURCE),
            ],
            check=True,
        )
        compiled = (build_dir / "BoundaryPostfixProbe.class").read_bytes()
        monitor_compiled = (build_dir / "BoundaryMonitorProbe.class").read_bytes()
    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT.write_bytes(parse_and_patch(compiled))
    MONITOR_OUTPUT.write_bytes(monitor_compiled)
    print(f"wrote {OUTPUT.relative_to(ROOT)} ({OUTPUT.stat().st_size} bytes)")
    print(f"wrote {MONITOR_OUTPUT.relative_to(ROOT)} ({MONITOR_OUTPUT.stat().st_size} bytes)")


if __name__ == "__main__":
    main()
