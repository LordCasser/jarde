#!/usr/bin/env python3
"""Freeze a Java 8 class with two allocation BCIs targeting the same anonymous class."""

from pathlib import Path
import hashlib
import shutil
import struct
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parent


def u2(data: bytes, offset: int) -> int:
    return struct.unpack_from(">H", data, offset)[0]


def u4(data: bytes, offset: int) -> int:
    return struct.unpack_from(">I", data, offset)[0]


def pool(data: bytes):
    entries = {}
    offset = 10
    index = 1
    while index < u2(data, 8):
        tag = data[offset]
        offset += 1
        if tag == 1:
            length = u2(data, offset)
            entries[index] = (tag, data[offset + 2 : offset + 2 + length])
            offset += 2 + length
        elif tag in (3, 4):
            offset += 4
        elif tag in (5, 6):
            offset += 8
            index += 1
        elif tag in (7, 8, 16, 19, 20):
            entries[index] = (tag, u2(data, offset))
            offset += 2
        elif tag in (9, 10, 11, 12, 17, 18):
            entries[index] = (tag, u2(data, offset), u2(data, offset + 2))
            offset += 4
        elif tag == 15:
            offset += 3
        else:
            raise ValueError(f"unexpected constant pool tag {tag}")
        index += 1
    return entries, offset


def skip_member(data: bytes, offset: int) -> int:
    count = u2(data, offset + 6)
    offset += 8
    for _ in range(count):
        offset += 6 + u4(data, offset + 2)
    return offset


def patch_caller(data: bytes) -> bytes:
    entries, offset = pool(data)

    def class_name(index: int) -> bytes:
        entry = entries[index]
        assert entry[0] == 7
        return entries[entry[1]][1]

    def constructor_ref(owner: bytes) -> int:
        found = []
        for index, entry in entries.items():
            if entry[0] != 10 or class_name(entry[1]) != owner:
                continue
            name_type = entries[entry[2]]
            assert name_type[0] == 12
            if entries[name_type[1]][1] == b"<init>" and entries[name_type[2]][1] == b"()V":
                found.append(index)
        assert len(found) == 1, (owner, found)
        return found[0]

    first = b"AnonymousDoubleSite$1"
    second = b"AnonymousDoubleSite$2"
    first_class = [index for index, entry in entries.items() if entry[0] == 7 and class_name(index) == first]
    second_class = [index for index, entry in entries.items() if entry[0] == 7 and class_name(index) == second]
    assert len(first_class) == len(second_class) == 1
    first_ctor = constructor_ref(first)
    second_ctor = constructor_ref(second)

    offset += 6
    interface_count = u2(data, offset)
    offset += 2 + 2 * interface_count
    field_count = u2(data, offset)
    offset += 2
    for _ in range(field_count):
        offset = skip_member(data, offset)
    method_count = u2(data, offset)
    offset += 2
    patched = bytearray(data)
    changed_new = changed_ctor = 0
    for _ in range(method_count):
        method_name = entries[u2(data, offset + 2)][1]
        attribute_count = u2(data, offset + 6)
        offset += 8
        for _ in range(attribute_count):
            attribute_name = entries[u2(data, offset)][1]
            length = u4(data, offset + 2)
            info = offset + 6
            if method_name == b"create" and attribute_name == b"Code":
                code_start = info + 8
                code_end = code_start + u4(data, info + 4)
                for at in range(code_start, code_end - 2):
                    if data[at : at + 3] == b"\xbb" + second_class[0].to_bytes(2, "big"):
                        patched[at + 1 : at + 3] = first_class[0].to_bytes(2, "big")
                        changed_new += 1
                    if data[at : at + 3] == b"\xb7" + second_ctor.to_bytes(2, "big"):
                        patched[at + 1 : at + 3] = first_ctor.to_bytes(2, "big")
                        changed_ctor += 1
            offset += 6 + length
    assert changed_new == changed_ctor == 1, (changed_new, changed_ctor)
    return bytes(patched)


with tempfile.TemporaryDirectory(prefix="jarde-anonymous-double-site-") as scratch:
    classes = Path(scratch)
    subprocess.run(["javac", "--release", "8", "-g", "-d", str(classes), str(ROOT / "AnonymousDoubleSite.java")], check=True)
    caller = classes / "AnonymousDoubleSite.class"
    caller.write_bytes(patch_caller(caller.read_bytes()))
    output = subprocess.run(["java", "-Xverify:all", "-cp", str(classes), "AnonymousDoubleSite"], check=True, text=True, capture_output=True)
    assert output.stdout == "first=11\nsecond=11\nsameClass=true\n", output.stdout
    for source in sorted(classes.glob("*.class")):
        target = ROOT / source.name
        shutil.copyfile(source, target)
        print(hashlib.sha256(target.read_bytes()).hexdigest(), target.name)
    print(output.stdout, end="")
