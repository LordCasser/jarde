#!/usr/bin/env python3
"""Duplicate the DOG key while preserving a verifier-valid javac helper class."""
from hashlib import sha256
from pathlib import Path
from zipfile import ZIP_STORED, ZipFile, ZipInfo

ROOT = Path(__file__).resolve().parent
SOURCE = ROOT / "grouped-input.jar"
TARGET = ROOT / "negative-input.jar"
EXPECTED_SOURCE_SHA256 = "8df2088c16499b1086ad9e65f516d166603be26d67c229083f5bbefd87fd3ea5"
EXPECTED_DOG_KEY_BCI = 33

if sha256(SOURCE.read_bytes()).hexdigest() != EXPECTED_SOURCE_SHA256:
    raise SystemExit("the frozen positive jar has changed")


def u2(data, at):
    return int.from_bytes(data[at : at + 2], "big")


def u4(data, at):
    return int.from_bytes(data[at : at + 4], "big")


def patch_helper(raw):
    data = bytearray(raw)
    cp = [b""] * u2(data, 8)
    at, index = 10, 1
    while index < len(cp):
        tag = data[at]
        at += 1
        if tag == 1:
            size = u2(data, at)
            cp[index] = bytes(data[at + 2 : at + 2 + size])
            at += 2 + size
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
            raise SystemExit(f"unexpected constant-pool tag {tag}")
        index += 1
    at += 6
    at += 2 + 2 * u2(data, at)
    fields = u2(data, at)
    at += 2
    for _ in range(fields):
        attributes = u2(data, at + 6)
        at += 8
        for _ in range(attributes):
            at += 6 + u4(data, at + 2)
    methods = u2(data, at)
    at += 2
    found = 0
    for _ in range(methods):
        name = cp[u2(data, at + 2)]
        attributes = u2(data, at + 6)
        at += 8
        for _ in range(attributes):
            attribute_name = cp[u2(data, at)]
            length = u4(data, at + 2)
            if name == b"<clinit>" and attribute_name == b"Code":
                code_start = at + 6 + 8
                if data[code_start + EXPECTED_DOG_KEY_BCI] != 0x05:
                    raise SystemExit("expected DOG's iconst_2 at BCI 33")
                data[code_start + EXPECTED_DOG_KEY_BCI] = 0x04
                found += 1
            at += 6 + length
    if found != 1:
        raise SystemExit("expected one helper <clinit> Code attribute")
    return bytes(data)


with ZipFile(SOURCE) as source:
    entries = {name: source.read(name) for name in source.namelist()}
entries["grouped/Subject$1.class"] = patch_helper(entries["grouped/Subject$1.class"])
with ZipFile(TARGET, "w", compression=ZIP_STORED) as target:
    for name in sorted(entries):
        info = ZipInfo(name, date_time=(1980, 1, 1, 0, 0, 0))
        info.compress_type = ZIP_STORED
        target.writestr(info, entries[name])
print(sha256(TARGET.read_bytes()).hexdigest())
