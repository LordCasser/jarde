"""Build five deliberately invalid member-construction controls from Java 8 classes.

The script checks every expected bytecode shape before changing bytes. It only
writes within this evidence directory and compiles in a private temporary tree.
"""

from __future__ import annotations

import hashlib
import struct
import subprocess
import tempfile
import zipfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
SIMPLE = HERE.parent.parent
BASE = SIMPLE / "classes" / "nested"
FIXTURE = HERE / "fixture"


def u2(data: bytes, offset: int) -> int:
    return struct.unpack_from(">H", data, offset)[0]


def u4(data: bytes, offset: int) -> int:
    return struct.unpack_from(">I", data, offset)[0]


def cp_and_members(data: bytes):
    assert data[:4] == b"\xca\xfe\xba\xbe"
    pos = 8
    count = u2(data, pos)
    pos += 2
    utf8 = {}
    classes = {}
    index = 1
    while index < count:
        tag = data[pos]
        pos += 1
        if tag == 1:
            size = u2(data, pos)
            pos += 2
            utf8[index] = data[pos : pos + size]
            pos += size
        elif tag in (3, 4, 9, 10, 11, 12, 17, 18):
            pos += 4
        elif tag in (5, 6):
            pos += 8
            index += 1
        elif tag in (7, 8, 16, 19, 20):
            if tag == 7:
                classes[index] = u2(data, pos)
            pos += 2
        elif tag == 15:
            pos += 3
        else:
            raise ValueError(f"unknown constant-pool tag {tag}")
        index += 1
    pos += 6  # access_flags, this_class, super_class
    interfaces = u2(data, pos)
    pos += 2 + 2 * interfaces

    def attrs(offset: int):
        entries = []
        count = u2(data, offset)
        offset += 2
        for _ in range(count):
            start = offset
            name = utf8[u2(data, offset)]
            size = u4(data, offset + 2)
            offset += 6 + size
            entries.append((name, start, offset))
        return entries, offset

    tables = []
    for _ in range(2):  # fields, methods
        members = []
        count = u2(data, pos)
        pos += 2
        for _ in range(count):
            name = utf8[u2(data, pos + 2)]
            descriptor = utf8[u2(data, pos + 4)]
            pos += 6
            attributes, pos = attrs(pos)
            members.append((name, descriptor, attributes))
        tables.append(members)
    class_attributes, pos = attrs(pos)
    assert pos == len(data)
    return utf8, classes, tables[1], class_attributes


def code_attr(data: bytes, method: bytes):
    _, _, methods, _ = cp_and_members(data)
    matches = [attrs for name, _, attrs in methods if name == method]
    assert len(matches) == 1
    codes = [(start, end) for name, start, end in matches[0] if name == b"Code"]
    assert len(codes) == 1
    start, end = codes[0]
    code_start = start + 14
    code_size = u4(data, start + 10)
    return start, end, code_start, code_start + code_size


def change_identity(data: bytes) -> bytes:
    _, _, begin, end = code_attr(data, b"make")
    code = data[begin:end]
    assert code[3:6] == bytes((0x59, 0x2B, 0x59)), code.hex()
    assert code[6] == 0xB8 and code[9] == 0x57
    patched = bytearray(data)
    patched[begin + 5] = 0x2A  # aload_0 is checked; aload_1 remains the physical outer
    return bytes(patched)


def change_identity_preserving_check(data: bytes) -> bytes:
    attr_start, _, begin, end = code_attr(data, b"make")
    code = data[begin:end]
    assert len(code) == 20 and code[3:6] == b"\x59\x2b\x59"
    assert code[6] == 0xB8 and code[9] == 0x57
    # Keep the exact aload; dup; requireNonNull; pop check, but check local 0.
    # Discard that checked copy, then pass local 1 as the physical outer.
    reordered = code[:4] + b"\x2a" + code[5:10] + b"\x57\x2b" + code[10:]
    assert len(reordered) == len(code) + 2
    patched = bytearray(data)
    struct.pack_into(">I", patched, attr_start + 2, u4(data, attr_start + 2) + 2)
    struct.pack_into(">I", patched, attr_start + 10, len(reordered))
    patched[begin:end] = reordered
    cp_and_members(bytes(patched))
    return bytes(patched)


def change_late(data: bytes) -> bytes:
    attr_start, _, begin, end = code_attr(data, b"make")
    code = data[begin:end]
    assert len(code) == 20 and code[3:6] == b"\x59\x2a\x59"
    assert code[6] == 0xB8 and code[9] == 0x57
    assert code[10] == 0x12 and code[12] == 0x1B and code[13] == 0xB8
    assert code[16] == 0xB7 and code[19] == 0xB0
    # Preserve physical first arg and ordinary arg; swap temporarily exposes outer to the check.
    reordered = code[:5] + code[10:16] + b"\x5f\x59" + code[6:10] + b"\x5f" + code[16:]
    assert len(reordered) == len(code) + 2
    patched = bytearray(data)
    struct.pack_into(">I", patched, attr_start + 2, u4(data, attr_start + 2) + 2)
    struct.pack_into(">I", patched, attr_start + 10, len(reordered))
    patched[begin:end] = reordered
    cp_and_members(bytes(patched))
    return bytes(patched)


def change_relation(data: bytes) -> bytes:
    utf8, classes, _, attributes = cp_and_members(data)
    matches = [(start, end) for name, start, end in attributes if name == b"InnerClasses"]
    assert len(matches) == 1
    start, end = matches[0]
    pos = start + 6
    count = u2(data, pos)
    pos += 2
    changed = bytearray(data)
    found = 0
    for _ in range(count):
        inner_index = u2(data, pos)
        if inner_index and utf8[classes[inner_index]] == b"nested/SimpleOuter$Inner":
            assert u2(data, pos + 2) != 0
            changed[pos + 2 : pos + 4] = b"\x00\x00"  # contradict member outer relation
            found += 1
        pos += 8
    assert pos == end and found == 1
    return bytes(changed)


def jar(name: str, members: dict[str, bytes]) -> None:
    with zipfile.ZipFile(HERE / name, "w", zipfile.ZIP_STORED) as archive:
        for path, content in sorted(members.items()):
            entry = zipfile.ZipInfo(path, date_time=(1980, 1, 1, 0, 0, 0))
            entry.compress_type = zipfile.ZIP_STORED
            archive.writestr(entry, content)


def main() -> None:
    with tempfile.TemporaryDirectory(prefix="jarde-inner-invalid-controls-") as temp:
        out = Path(temp)
        subprocess.run(
            ["javac", "--release", "8", "-g:none", "-Xlint:-options", "-cp", str(SIMPLE / "classes"), "-d", str(out), *map(str, sorted(FIXTURE.glob("*.java")))],
            check=True,
        )
        outer = (BASE / "SimpleOuter.class").read_bytes()
        inner = (BASE / "SimpleOuter$Inner.class").read_bytes()
        caller = (BASE / "UseInner.class").read_bytes()
        runner = (BASE / "InnerRunner.class").read_bytes()
        identity = (out / "nested/IdentityUse.class").read_bytes()
        identity_runner = (out / "nested/IdentityRunner.class").read_bytes()
        late = (out / "nested/LateUse.class").read_bytes()
        late_runner = (out / "nested/LateRunner.class").read_bytes()
        baselines = {
            "identity-source": {"SimpleOuter.class": outer, "SimpleOuter$Inner.class": inner, "IdentityUse.class": identity, "IdentityRunner.class": identity_runner},
            "late-source": {"SimpleOuter.class": outer, "SimpleOuter$Inner.class": inner, "LateUse.class": late, "LateRunner.class": late_runner},
        }
        variants = {
            "wrong-relation": ({"SimpleOuter.class": outer, "SimpleOuter$Inner.class": change_relation(inner), "UseInner.class": caller, "InnerRunner.class": runner}, "SimpleOuter$Inner.class"),
            "wrong-outer-relation": ({"SimpleOuter.class": change_relation(outer), "SimpleOuter$Inner.class": inner, "UseInner.class": caller, "InnerRunner.class": runner}, "SimpleOuter.class"),
            "wrong-identity": ({"SimpleOuter.class": outer, "SimpleOuter$Inner.class": inner, "IdentityUse.class": change_identity(identity), "IdentityRunner.class": identity_runner}, "IdentityUse.class"),
            "wrong-identity-checked": ({"SimpleOuter.class": outer, "SimpleOuter$Inner.class": inner, "IdentityUse.class": change_identity_preserving_check(identity), "IdentityRunner.class": identity_runner}, "IdentityUse.class"),
            "late-check": ({"SimpleOuter.class": outer, "SimpleOuter$Inner.class": inner, "LateUse.class": change_late(late), "LateRunner.class": late_runner}, "LateUse.class"),
        }
        lines = []
        for name, files in baselines.items():
            jar(name + ".jar", {"nested/" + key: value for key, value in files.items()})
            lines.append(f"{hashlib.sha256((HERE / (name + '.jar')).read_bytes()).hexdigest()}  {name}.jar")
        for name, (files, modified) in variants.items():
            jar(name + ".jar", {"nested/" + key: value for key, value in files.items()})
            (HERE / (name + ".class")).write_bytes(files[modified])
            for key, data in files.items():
                lines.append(f"{hashlib.sha256(data).hexdigest()}  {name}:nested/{key}")
            lines.append(f"{hashlib.sha256((HERE / (name + '.jar')).read_bytes()).hexdigest()}  {name}.jar")
        (HERE / "SHA256.txt").write_text("\n".join(lines) + "\n")


if __name__ == "__main__":
    main()
