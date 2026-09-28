#!/usr/bin/env python3
"""Freeze local, verifier-valid counterexamples to the fixed Test3 certificate."""

import hashlib
import pathlib
import struct
import subprocess
import tempfile

ROOT = pathlib.Path(__file__).resolve().parent
SOURCE = ROOT / "TestTryCatchFinally3$TestCls.class"
CLASS = "jadx.tests.integration.trycatch.TestTryCatchFinally3$TestCls"


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
    assert hashlib.sha256(original).hexdigest() == "9c3dc61868a81eb87f90228c5ddad4336ca4ae540621e5cb9b1589474c90e71e"
    code, rows, code_attribute, stackmap = method_code(original)
    assert [original[code + bci] for bci in (35, 38, 39, 42, 45, 53, 58, 59, 62, 65, 67, 68, 71, 73)] == [
        0xA7, 0x2A, 0xB6, 0xA7, 0x4D, 0xB9, 0x2A, 0xB6, 0xA7, 0x3A, 0x2A, 0xB6, 0x19, 0xBF
    ]
    assert u2(original, rows - 2) == 4
    assert [(u2(original, rows + i), u2(original, rows + i + 2), u2(original, rows + i + 4))
            for i in (0, 8, 16, 24)] == [(0, 38, 45), (0, 38, 65), (45, 58, 65), (65, 67, 65)]
    result = {}

    data = original.copy()
    data[code + 58] = 0x01  # aconst_null is a valid ClassNode receiver but throws NPE
    result["wrong-receiver"] = data

    data = original.copy()
    struct.pack_into(">H", data, code + 60, 7)  # ClassNode.load:()V, not unload
    result["wrong-target"] = data

    data = original.copy()
    struct.pack_into(">H", data, rows + 24 + 2, 71)  # self-row now covers unload
    result["self-row-covers-cleanup"] = data

    data = original.copy()
    data[code + 71 : code + 73] = b"\x01\x00"  # null then nop; athrow makes NPE
    result["throwable-rewritten"] = data

    data = original.copy()
    struct.pack_into(">h", data, code + 43, 0 - 42)  # cleanup re-enters the protected body
    frames = stackmap + 8
    assert u2(data, stackmap + 6) == 5
    assert data[frames] == 252 and u2(data, frames + 1) == 11
    data[frames:frames] = b"\x00"  # explicit same_frame at the backward branch target BCI 0
    struct.pack_into(">H", data, stackmap + 6, 6)
    struct.pack_into(">H", data, frames + 2, 10)  # old BCI 11 is now relative to BCI 0
    struct.pack_into(">I", data, stackmap + 2, u4(data, stackmap + 2) + 1)
    struct.pack_into(">I", data, code_attribute + 2, u4(data, code_attribute + 2) + 1)
    result["extra-protected-entry"] = data

    data = original.copy()
    struct.pack_into(">H", data, rows + 16, 42)  # catch-all also covers prior transfer
    result["extra-catch-coverage"] = data

    data = original.copy()
    marker = b"\x00\x08iterator"
    assert data.count(marker) == 1
    at = data.index(marker) + 2
    data[at : at + 8] = b"listIter"  # unrelated symbolic call; verifier defers resolution
    result["unknown-body-invoke"] = data
    return result


def main():
    output = ROOT / "neighbors"
    with tempfile.TemporaryDirectory(prefix="cf16-test3-neighbors-") as temp:
        classes = pathlib.Path(temp) / "classes"
        classes.mkdir()
        loader = pathlib.Path(temp) / "VerifyLoad.java"
        loader.write_text('public final class VerifyLoad { public static void main(String[] args) throws Exception { Class.forName(args[0]); } }\n')
        sources = list((ROOT / "standins").rglob("*.java")) + [loader]
        subprocess.run(["javac", "--release", "8", "-g:none", "-Xlint:-options", "-d", str(classes), *map(str, sources)], check=True)
        target = classes / "jadx/tests/integration/trycatch/TestTryCatchFinally3$TestCls.class"
        target.parent.mkdir(parents=True)
        manifest = []
        for name, data in variants().items():
            path = output / (name + ".class")
            path.parent.mkdir(exist_ok=True)
            target.write_bytes(data)
            subprocess.run(["java", "-Xverify:all", "-cp", str(classes), "VerifyLoad", CLASS], check=True)
            path.write_bytes(data)
            manifest.append(f"{name} {hashlib.sha256(data).hexdigest()} java -Xverify:all OK")
        (output / "sha256-and-verify.txt").write_text("\n".join(manifest) + "\n")
        print("\n".join(manifest))


if __name__ == "__main__":
    main()
