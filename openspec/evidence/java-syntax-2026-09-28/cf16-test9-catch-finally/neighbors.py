#!/usr/bin/env python3
"""Freeze local, verifier-valid counterexamples to the fixed Test9 nullable-resource certificate."""

import hashlib
import pathlib
import struct
import subprocess
import tempfile

ROOT = pathlib.Path(__file__).resolve().parent
SOURCE = ROOT / "TestTryCatchFinally9$TestCls.class"
CLASS = "jadx.tests.integration.trycatch.TestTryCatchFinally9$TestCls"


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
            names[index] = bytes(data[at + 2 : at + 2 + size]).decode()
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
                count = u2(data, nested - 2)
                for _ in range(count):
                    if names[u2(data, nested)] == "StackMapTable":
                        return code, rows, at, nested
                    nested += 6 + u4(data, nested + 2)
                raise ValueError("test StackMapTable missing")
            at += 6 + length
    raise ValueError("test Code attribute missing")


def variants():
    original = bytearray(SOURCE.read_bytes())
    assert hashlib.sha256(original).hexdigest() == "251e21b8ade9f6a9096d807dc31a9fcaae4e2a551600a7230b8f1b7f9f165381"
    code, rows, code_attribute, stackmap = method_code(original)
    assert [(u2(original, rows + i), u2(original, rows + i + 2), u2(original, rows + i + 4))
            for i in (0, 8)] == [(2, 43, 53), (53, 55, 53)]
    assert [original[code + bci] for bci in (37, 44, 47, 48, 51, 56, 59, 60, 63)] == [
        0xA7, 0xC6, 0x2B, 0xB6, 0x2D, 0xC6, 0x2B, 0xB6, 0x19
    ]
    result = {}

    data = original.copy()
    data[code + 44] = 0xC7  # normal copy tests non-null instead
    result["different-null-test"] = data

    data = original.copy()
    data[code + 47] = 0x01  # null receiver in normal copy
    result["different-receiver"] = data

    data = original.copy()
    data[code + 51] = 0x01  # return null rather than saved local 3
    result["saved-return-rewritten"] = data

    data = original.copy()
    data[code + 63 : code + 65] = b"\x01\x00"  # null; nop; athrow at 65
    result["throwable-rewritten"] = data

    data = original.copy()
    struct.pack_into(">H", data, rows + 8 + 2, 63)  # self-row now covers close
    result["self-row-covers-close"] = data

    data = original.copy()
    data[code + 37 : code + 40] = b"\xb0\x00\x00"  # extra early areturn skips close
    first_frame = stackmap + 8
    assert data[first_frame] == 253 and u2(data, first_frame + 1) == 40
    struct.pack_into(">H", data, first_frame + 1, 38)
    data[first_frame + 9:first_frame + 9] = b"\x01"  # same_frame at BCI 40
    struct.pack_into(">H", data, stackmap + 6, u2(data, stackmap + 6) + 1)
    struct.pack_into(">I", data, stackmap + 2, u4(data, stackmap + 2) + 1)
    struct.pack_into(">I", data, code_attribute + 2, u4(data, code_attribute + 2) + 1)
    result["extra-normal-exit"] = data

    # Append a distinct invokevirtual InputStream.reset:()V and redirect one copy only.
    data = original.copy()
    at = 10
    pool_count = u2(data, 8)
    index = 1
    while index < pool_count:
        tag = data[at]
        at += 1
        if tag == 1:
            at += 2 + u2(data, at)
        elif tag in (3, 4, 9, 10, 11, 12, 17, 18):
            at += 4
        elif tag in (5, 6):
            at += 8
            index += 1
        elif tag in (7, 8, 16, 19, 20):
            at += 2
        elif tag == 15:
            at += 3
        else:
            raise ValueError(tag)
        index += 1
    extra = b"\x01\x00\x05reset" + b"\x0c" + struct.pack(">HH", pool_count, 6) \
        + b"\x0a" + struct.pack(">HH", 41, pool_count + 1)
    data[at:at] = extra
    struct.pack_into(">H", data, 8, pool_count + 3)
    code, _, _, _ = method_code(data)
    struct.pack_into(">H", data, code + 61, pool_count + 2)  # handler copy only
    result["different-target"] = data
    return result


def main():
    output = ROOT / "neighbors"
    with tempfile.TemporaryDirectory(prefix="cf16-test9-neighbors-") as temp:
        classes = pathlib.Path(temp) / "classes"
        classes.mkdir()
        loader = pathlib.Path(temp) / "VerifyLoad.java"
        loader.write_text('public final class VerifyLoad { public static void main(String[] args) throws Exception { Class.forName(args[0]); } }\n')
        subprocess.run(["javac", "--release", "8", "-g:none", "-Xlint:-options", "-d", str(classes), str(loader)], check=True)
        target = classes / "jadx/tests/integration/trycatch/TestTryCatchFinally9$TestCls.class"
        target.parent.mkdir(parents=True)
        manifest = []
        for name, data in variants().items():
            target.write_bytes(data)
            subprocess.run(["java", "-Xverify:all", "-cp", str(classes), "VerifyLoad", CLASS], check=True)
            path = output / (name + ".class")
            path.parent.mkdir(exist_ok=True)
            path.write_bytes(data)
            manifest.append(f"{name} {hashlib.sha256(data).hexdigest()} java -Xverify:all OK")
        (output / "sha256-and-verify.txt").write_text("\n".join(manifest) + "\n")
        print("\n".join(manifest))


if __name__ == "__main__":
    main()
