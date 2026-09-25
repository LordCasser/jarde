#!/usr/bin/env python3
"""Swap the two verified enum-map integers without editing names or the switch body."""

from hashlib import sha256
from pathlib import Path
from shutil import copy2


ROOT = Path(__file__).resolve().parent
SOURCE = ROOT / "original"
TARGET = ROOT / "patched-map"
EXPECTED_SHA256 = "5cbf19d12db909aa88ad1a504f6d8f8a096f6d9622cae972f0569b0b591aabd6"


def u2(data, at):
    return int.from_bytes(data[at : at + 2], "big")


def u4(data, at):
    return int.from_bytes(data[at : at + 4], "big")


def skip_attributes(data, at, count):
    for _ in range(count):
        at += 6 + u4(data, at + 2)
    return at


def patch(data):
    if sha256(data).hexdigest() != EXPECTED_SHA256:
        raise ValueError("the frozen synthetic class has changed")
    cp = [b""] * u2(data, 8)
    at = 10
    index = 1
    while index < len(cp):
        tag = data[at]
        at += 1
        if tag == 1:
            length = u2(data, at)
            cp[index] = bytes(data[at + 2 : at + 2 + length])
            at += 2 + length
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
            raise ValueError(f"unexpected constant pool tag {tag}")
        index += 1
    at += 6  # access, this, super
    at += 2 + 2 * u2(data, at)  # interfaces
    fields = u2(data, at)
    at += 2
    for _ in range(fields):
        at = skip_attributes(data, at + 8, u2(data, at + 6))
    methods = u2(data, at)
    at += 2
    found = False
    for _ in range(methods):
        name = cp[u2(data, at + 2)]
        attributes = u2(data, at + 6)
        at += 8
        for _ in range(attributes):
            attribute_name = cp[u2(data, at)]
            length = u4(data, at + 2)
            if name == b"<clinit>" and attribute_name == b"Code":
                code = at + 6 + 8
                if (data[code + 18], data[code + 33]) != (0x04, 0x05):
                    raise ValueError("the two mapped values are no longer iconst_1/iconst_2")
                data[code + 18], data[code + 33] = 0x05, 0x04
                found = True
            at += 6 + length
    if not found:
        raise ValueError("the synthetic initializer was not found")


if __name__ == "__main__":
    TARGET.mkdir(exist_ok=True)
    for source in SOURCE.glob("*.class"):
        copy2(source, TARGET / source.name)
    subject = TARGET / "EnumSwitchSubject$1.class"
    data = bytearray(subject.read_bytes())
    patch(data)
    subject.write_bytes(data)
    print(sha256(data).hexdigest())
