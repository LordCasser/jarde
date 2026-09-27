#!/usr/bin/env python3
"""Freeze verifier-valid one-fact mutations of the fixed Test17 probe class."""

import hashlib
import pathlib
import struct
import subprocess
import sys


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
    interfaces = u2(data, at)
    at += 2
    for _ in range(interfaces):
        at += 2
    fields = u2(data, at)
    at += 2
    for _ in range(fields):
        attributes = u2(data, at + 6)
        at += 6
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
                code = at + 6 + 8
                rows = code + u4(data, at + 6 + 4) + 2
                assert u4(data, at + 6 + 4) == 32
                assert u2(data, rows - 2) == 4
                return code, rows, at
            at += 6 + length
    raise ValueError("test()I Code missing")


def main():
    source, output = map(pathlib.Path, sys.argv[1:])
    target = pathlib.Path("jadx/tests/integration/trycatch/TestTryCatchFinally17$TestCls.class")
    original = bytearray((source / target).read_bytes())
    code, rows, code_attribute = method_code(original)
    assert [original[code + bci] for bci in (0, 3, 9, 10, 16, 17, 18, 19, 22, 23, 24, 25, 28, 29, 30, 31)] == [0xb8, 0xb8, 0x4c, 0xb8, 0x4c, 0x04, 0x3d, 0xb8, 0x1c, 0xac, 0x4e, 0xb8, 0x2d, 0xbf, 0x03, 0xac]
    assert original[rows : rows + 32].hex() == "000000030009000f000000030010001100000003001800000010001300180000"
    variants = {}

    data = original.copy()
    data[code + 20 : code + 22] = b"\x00\x07"  # doSomething()V instead of doFinally()V
    variants["different-target"] = data

    data = original.copy()
    data[rows + 24 + 2 : rows + 24 + 4] = b"\x00\x16"  # second catch-all protects its own cleanup
    variants["cleanup-self-protected"] = data

    data = original.copy()
    data[rows + 16 + 2 : rows + 16 + 4] = b"\x00\x06"  # body catch-all also protects normal cleanup
    variants["cleanup-covered"] = data

    data = original.copy()
    data[rows : rows + 8], data[rows + 8 : rows + 16] = data[rows + 8 : rows + 16], data[rows : rows + 8]
    variants["rows-swapped"] = data

    data = original.copy()
    data[code + 17] = 0x03  # saved return becomes 0
    variants["saved-return-rewritten"] = data

    data = original.copy()
    data[code + 28] = 0x01  # aconst_null; athrow now throws NPE instead of original Throwable
    variants["throwable-rewritten"] = data

    data = original.copy()
    data[code + 14 : code + 16] = struct.pack(">h", -10)  # first catch's goto enters normal cleanup at BCI 3
    nested = rows + 32
    attributes = u2(data, nested)
    nested += 2
    for _ in range(attributes):
        length = u4(data, nested + 2)
        payload = nested + 6
        if data[payload : payload + 3] == bytes.fromhex("000449"):
            # Add the verifier's empty-stack frame at BCI 3. The former BCI 9
            # frame moves after it, so its delta changes from 9 to 5.
            frames = bytearray(data[payload : payload + length])
            frames[:3] = bytes.fromhex("00050345")
            data[payload : payload + length] = frames
            struct.pack_into(">I", data, nested + 2, length + 1)
            struct.pack_into(">I", data, code_attribute + 2, u4(data, code_attribute + 2) + 1)
            variants["external-cleanup-entry"] = data
            break
        nested += 6 + length
    else:
        raise ValueError("fixed StackMapTable changed")

    for name, data in variants.items():
        directory = output / name
        path = directory / target
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)
        # Keep probe auxiliary classes and Runner unchanged.
        for sibling in (source / target.parent).glob("*.class"):
            if sibling.name != target.name:
                (directory / target.parent / sibling.name).write_bytes(sibling.read_bytes())
        command = ["java", "-Xverify:all", "-cp", str(directory), "jadx.tests.integration.trycatch.Runner"]
        result = subprocess.run(command, check=True, capture_output=True, text=True)
        (directory / "run.txt").write_text(result.stdout)
        javap = subprocess.run(["javap", "-classpath", str(directory), "-p", "-c", "-v", "jadx.tests.integration.trycatch.TestTryCatchFinally17$TestCls"], check=True, capture_output=True, text=True)
        method = javap.stdout.split("  public int test();", 1)[1].split("      LineNumberTable:", 1)[0]
        (directory / "test.javap.txt").write_text(method)
        print(name, hashlib.sha256(data).hexdigest(), "verified", result.stdout.splitlines()[0])


if __name__ == "__main__":
    main()
