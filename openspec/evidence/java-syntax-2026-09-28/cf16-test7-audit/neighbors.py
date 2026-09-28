#!/usr/bin/env python3
"""Freeze verifier-valid mutations around the fixed four-row Test7 method."""

import hashlib
from pathlib import Path
import re
import struct
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parent
SOURCE = ROOT / "baseline/debug/physical-class/TestTryCatchFinally7$TestCls.class"
CLASS = "jadx.tests.integration.trycatch.TestTryCatchFinally7$TestCls"


def u2(data, at):
    return struct.unpack_from(">H", data, at)[0]


def u4(data, at):
    return struct.unpack_from(">I", data, at)[0]


def method_code(data):
    at = 8
    count = u2(data, at)
    at += 2
    names = {}
    for index in range(1, count):
        tag = data[at]
        at += 1
        if tag == 1:
            size = u2(data, at)
            names[index] = bytes(data[at + 2:at + 2 + size]).decode()
            at += 2 + size
        elif tag in (3, 4, 9, 10, 11, 12, 17, 18):
            at += 4
        elif tag in (5, 6):
            at += 8
        elif tag in (7, 8, 16, 19, 20):
            at += 2
        elif tag == 15:
            at += 3
        else:
            raise ValueError(tag)
    at += 6
    at += 2 * u2(data, at) + 2
    fields = u2(data, at)
    at += 2
    for _ in range(fields):
        at += 6
        attributes = u2(data, at)
        at += 2
        for _ in range(attributes):
            at += 6 + u4(data, at + 2)
    methods = u2(data, at)
    at += 2
    for _ in range(methods):
        name = names[u2(data, at + 2)]
        attributes = u2(data, at + 6)
        at += 8
        for _ in range(attributes):
            attribute = names[u2(data, at)]
            length = u4(data, at + 2)
            if name == "test" and attribute == "Code":
                code = at + 14
                rows = code + u4(data, at + 10) + 2
                nested = rows + 8 * u2(data, rows - 2) + 2
                for _ in range(u2(data, nested - 2)):
                    if names[u2(data, nested)] == "StackMapTable":
                        return code, rows, at, nested
                    nested += 6 + u4(data, nested + 2)
                raise ValueError("StackMapTable missing")
            at += 6 + length
    raise ValueError("test method missing")


def variants(classes):
    original = bytearray(SOURCE.read_bytes())
    assert hashlib.sha256(original).hexdigest() == "679d3163c709f59f0f9cf5db91f82ce142f1f18fa0f64cdabd95da2798e3d604"
    code, rows, code_attr, stackmap = method_code(original)
    assert [original[code + bci] for bci in (6, 8, 12, 13, 16, 21, 32, 35, 37, 42, 47, 50, 51)] == [
        0x2a, 0xb4, 0x60, 0xb5, 0xa7, 0x3d, 0xa7, 0x3a, 0x2a, 0x04, 0x19, 0x1c, 0xac
    ]
    assert [(u2(original, rows + i), u2(original, rows + i + 2), u2(original, rows + i + 4))
            for i in (0, 8, 16, 24)] == [(0, 6, 19), (0, 6, 35), (19, 22, 35), (35, 37, 35)]
    result = {}

    # A second declared field makes one cleanup copy a valid but different target.
    source = (ROOT / "TestTryCatchFinally7.java").read_text().replace(
        "private int f = 0;", "private int f = 0;\n\t\tprivate int g = 0;"
    )
    extra = classes.parent / "TestTryCatchFinally7.java"
    extra.write_text(source)
    subprocess.run(["javac", "--release", "8", "-g", "-d", str(classes), str(extra)], check=True)
    with_field = bytearray((classes / "jadx/tests/integration/trycatch/TestTryCatchFinally7$TestCls.class").read_bytes())
    listing = subprocess.check_output(["javap", "-v", "-p", "-classpath", str(classes), CLASS], text=True)
    field_index = int(re.search(r"^\s*#(\d+) = Fieldref\s+.*// .*\.g:I$", listing, re.M).group(1))
    field_code, _, _, _ = method_code(with_field)
    assert [with_field[field_code + bci] for bci in (0, 2, 5, 6, 8, 12, 13, 16, 19, 21, 22, 24, 28, 29, 32, 35, 37, 39, 43, 44, 47, 50, 51)] == [
        original[code + bci] for bci in (0, 2, 5, 6, 8, 12, 13, 16, 19, 21, 22, 24, 28, 29, 32, 35, 37, 39, 43, 44, 47, 50, 51)
    ]
    struct.pack_into(">H", with_field, field_code + 9, field_index)
    struct.pack_into(">H", with_field, field_code + 14, field_index)
    result["wrong-field"] = with_field

    data = original.copy()
    data[code + 37] = 0x01  # aconst_null: verifier accepts the receiver; getfield throws
    result["wrong-receiver"] = data

    data = original.copy()
    data[code + 42] = 0x05  # iconst_2
    result["wrong-increment"] = data

    data = original.copy()
    data[code + 50] = 0x03  # iconst_0 instead of reloading the saved result
    result["rewritten-return"] = data

    data = original.copy()
    data[code + 47:code + 49] = b"\x01\x00"  # null then nop; athrow changes identity
    result["rewritten-throwable"] = data

    data = original.copy()
    struct.pack_into(">H", data, rows + 24 + 2, 47)
    result["self-row-covers-cleanup"] = data

    data = original.copy()
    struct.pack_into(">h", data, code + 33, -32)  # catch cleanup re-enters protected BCI 0
    frame = stackmap + 8
    assert u2(data, stackmap + 6) == 3 and data[frame] == 83
    data[frame:frame] = b"\x00"  # explicit frame at the new backward target BCI 0
    data[frame + 1] = 82  # old frame at BCI 19 now follows frame 0
    struct.pack_into(">H", data, stackmap + 6, 4)
    struct.pack_into(">I", data, stackmap + 2, u4(data, stackmap + 2) + 1)
    struct.pack_into(">I", data, code_attr + 2, u4(data, code_attr + 2) + 1)
    result["extra-protected-entry"] = data
    return result


def main():
    output = ROOT / "neighbors"
    output.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="cf16-test7-neighbors-") as temp:
        classes = Path(temp) / "classes"
        classes.mkdir()
        loader = Path(temp) / "VerifyLoad.java"
        loader.write_text("public final class VerifyLoad { public static void main(String[] args) throws Exception { Class.forName(args[0]); } }\n")
        subprocess.run(["javac", "--release", "8", "-d", str(classes), str(loader)], check=True)
        target = classes / "jadx/tests/integration/trycatch/TestTryCatchFinally7$TestCls.class"
        target.parent.mkdir(parents=True, exist_ok=True)
        manifest = []
        for name, data in variants(classes).items():
            target.write_bytes(data)
            subprocess.run(["java", "-Xverify:all", "-cp", str(classes), "VerifyLoad", CLASS], check=True)
            (output / f"{name}.class").write_bytes(data)
            manifest.append(f"{name} {hashlib.sha256(data).hexdigest()} java -Xverify:all OK")
        (output / "sha256-and-verify.txt").write_text("\n".join(manifest) + "\n")
        print("\n".join(manifest))


if __name__ == "__main__":
    main()
