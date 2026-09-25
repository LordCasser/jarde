#!/usr/bin/env python3
"""Change one float constant named by FloatingArrays.floats' AnnotationDefault array."""
import struct
from pathlib import Path

HERE = Path(__file__).resolve().parent
SOURCE = HERE / "arrays-original" / "FloatingArrays.class"
TARGET = HERE / "arrays-payload-nan" / "FloatingArrays.class"


def u2(data, offset):
    return struct.unpack_from(">H", data, offset)[0]


def u4(data, offset):
    return struct.unpack_from(">I", data, offset)[0]


def main():
    data = bytearray(SOURCE.read_bytes())
    count = u2(data, 8)
    pool, payloads = {}, {}
    offset, index = 10, 1
    while index < count:
        tag = data[offset]
        offset += 1
        if tag == 1:
            size = u2(data, offset)
            offset += 2
            pool[index] = bytes(data[offset:offset + size]).decode("utf-8")
            offset += size
        elif tag in (3, 4):
            pool[index] = tag
            payloads[index] = offset
            offset += 4
        elif tag in (5, 6):
            pool[index] = tag
            payloads[index] = offset
            offset += 8
            index += 1
        elif tag in (7, 8, 16, 19, 20):
            pool[index] = u2(data, offset)
            offset += 2
        elif tag in (9, 10, 11, 12, 17, 18):
            offset += 4
        elif tag == 15:
            offset += 3
        else:
            raise ValueError(f"unsupported constant-pool tag {tag}")
        index += 1
    cp_end = offset
    offset += 6
    interfaces = u2(data, offset)
    offset += 2 + interfaces * 2
    fields = u2(data, offset)
    offset += 2
    for _ in range(fields):
        offset = skip_member(data, offset)
    methods = u2(data, offset)
    offset += 2
    target_index = None
    for _ in range(methods):
        name = pool[u2(data, offset + 2)]
        attributes = u2(data, offset + 6)
        offset += 8
        for _ in range(attributes):
            attr_name = pool[u2(data, offset)]
            size = u4(data, offset + 2)
            info = offset + 6
            if name == "floats" and attr_name == "AnnotationDefault":
                assert data[info] == ord("[")
                assert u2(data, info + 1) == 2
                assert data[info + 3] == ord("F")
                target_index = u2(data, info + 4)
            offset = info + size
    assert target_index is not None and pool[target_index] == 4
    payload = payloads[target_index]
    assert u4(data, payload) == 0x3f000000
    data[payload:payload + 4] = (0x7fc00001).to_bytes(4, "big")
    assert len(data) == SOURCE.stat().st_size
    assert data[cp_end:] == SOURCE.read_bytes()[cp_end:]
    TARGET.parent.mkdir(exist_ok=True)
    TARGET.write_bytes(data)


def skip_member(data, offset):
    attributes = u2(data, offset + 6)
    offset += 8
    for _ in range(attributes):
        offset += 6 + u4(data, offset + 2)
    return offset


if __name__ == "__main__":
    main()
