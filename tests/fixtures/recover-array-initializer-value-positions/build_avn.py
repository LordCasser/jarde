#!/usr/bin/env python3
"""Writes `AVN.class` — the hand-built negatives of `recover-array-initializer-value-positions`.

`javac` never emits a dance value with two consumers, so the shapes have to be stated by hand. The
class holds three static `()I` members over the *same* initialization dance
(`iconst_1; newarray int; dup; iconst_0; bipush 9; iastore`), differing in what follows it:

    single()      … ; iconst_0; iaload; ireturn        the value's one reader — the admitted shape
    twoReaders()  … ; dup; iconst_0; iaload; arraylength; iadd; ireturn
                                                       the value is duplicated: two readers
    discarded()   … ; dup; pop; iconst_0; iaload; ireturn
                                                       a second consumer that reads nothing

The first is the self-test of this builder (the bytes really are the anchor's shape, so a rendering
that writes `return new int[]{9}[0];` is evidence the bytes are what this file says). The other two
are the gate's negatives: the dance's value has a second consumer, so the initializer must stay
unproved and the method must keep the text it had before the change — byte for byte.

Run from this directory: `python3 build_avn.py`. The class file is written next to this script.
"""

import struct
from pathlib import Path

MAJOR = 52  # Java 8: no StackMapTable is needed for a straight-line body with no branch target.
# iconst_1; newarray int; dup; iconst_0; bipush 9; iastore — the dance, leaving the array on the stack.
DANCE = bytes([0x04, 0xBC, 0x0A, 0x59, 0x03, 0x10, 0x09, 0x4F])
MEMBERS = (
    (b"single", DANCE + bytes([0x03, 0x2E, 0xAC])),  # iconst_0; iaload; ireturn
    # dup; iconst_0; iaload; arraylength; iadd; ireturn
    (b"twoReaders", DANCE + bytes([0x59, 0x03, 0x2E, 0xBE, 0x60, 0xAC])),
    (b"discarded", DANCE + bytes([0x59, 0x57, 0x03, 0x2E, 0xAC])),  # dup; pop; iconst_0; iaload; ireturn
)


def utf8(value: bytes) -> bytes:
    return b"\x01" + struct.pack(">H", len(value)) + value


def class_entry(name_index: int) -> bytes:
    return b"\x07" + struct.pack(">H", name_index)


def code_attribute(code: bytes, max_stack: int, max_locals: int) -> bytes:
    body = struct.pack(">HHI", max_stack, max_locals, len(code)) + code + struct.pack(">HH", 0, 0)
    return struct.pack(">HI", 0x0001, len(body)) + body  # name_index 0x0001 is `Code`


def main() -> None:
    pool = [utf8(b"Code"), utf8(b"AVN"), class_entry(2), utf8(b"java/lang/Object"), class_entry(4)]
    member_indices = []
    for name, _ in MEMBERS:
        member_indices.append((len(pool) + 1, len(pool) + 2))
        pool.append(utf8(name))
        pool.append(utf8(b"()I"))
    out = bytearray(b"\xca\xfe\xba\xbe")
    out += struct.pack(">HH", 0, MAJOR)
    out += struct.pack(">H", len(pool) + 1)
    out += b"".join(pool)
    out += struct.pack(">HHH", 0x0021, 3, 5)  # public super, this AVN, super java/lang/Object
    out += struct.pack(">H", 0)  # no interfaces
    out += struct.pack(">H", 0)  # no fields
    out += struct.pack(">H", len(MEMBERS))
    for (name_index, descriptor_index), (_, code) in zip(member_indices, MEMBERS):
        out += struct.pack(">HHHH", 0x0009, name_index, descriptor_index, 1)  # public static ()I
        out += code_attribute(code, 4, 4)
    out += struct.pack(">H", 0)  # no class attributes
    Path(__file__).with_name("AVN.class").write_bytes(bytes(out))


if __name__ == "__main__":
    main()
