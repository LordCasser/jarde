#!/usr/bin/env python3
"""Make a verifier-valid table whose RED and BLUE entries share integer key 1."""

from hashlib import sha256
from pathlib import Path


ROOT = Path(__file__).resolve().parent
SOURCE = ROOT / "original" / "EnumSwitchSubject$1.class"
TARGET = ROOT / "negative" / "duplicate-key" / "classes" / "EnumSwitchSubject$1.class"
EXPECTED_SHA256 = "5cbf19d12db909aa88ad1a504f6d8f8a096f6d9622cae972f0569b0b591aabd6"


if sha256(SOURCE.read_bytes()).hexdigest() != EXPECTED_SHA256:
    raise SystemExit("the frozen synthetic class has changed")
data = bytearray(SOURCE.read_bytes())
if data[0:4] != b"\xca\xfe\xba\xbe":
    raise SystemExit("unexpected input class")

# Locate the unique Code array using the same small class-file walk as patch_swapped_map.py.
def u2(at):
    return int.from_bytes(data[at : at + 2], "big")


def u4(at):
    return int.from_bytes(data[at : at + 4], "big")


cp = [b""] * u2(8)
at = 10
index = 1
while index < len(cp):
    tag = data[at]
    at += 1
    if tag == 1:
        size = u2(at)
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
        raise SystemExit(f"unexpected constant pool tag {tag}")
    index += 1
at += 6
at += 2 + 2 * u2(at)
fields = u2(at)
at += 2
for _ in range(fields):
    count = u2(at + 6)
    at += 8
    for _ in range(count):
        at += 6 + u4(at + 2)
methods = u2(at)
at += 2
matches = 0
for _ in range(methods):
    name = cp[u2(at + 2)]
    attributes = u2(at + 6)
    at += 8
    for _ in range(attributes):
        attribute_name = cp[u2(at)]
        length = u4(at + 2)
        if name == b"<clinit>" and attribute_name == b"Code":
            code = at + 6 + 8
            if data[code + 33] != 0x05:
                raise SystemExit("expected BLUE's frozen `iconst_2` at BCI 33")
            data[code + 33] = 0x04
            matches += 1
        at += 6 + length
if matches != 1:
    raise SystemExit("expected one synthetic <clinit> Code attribute")
TARGET.write_bytes(data)
print(sha256(data).hexdigest())
