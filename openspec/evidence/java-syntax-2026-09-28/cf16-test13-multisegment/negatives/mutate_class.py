#!/usr/bin/env python3
"""Apply one verifier-oriented byte patch to the compiled Test13 probe."""
import struct
import sys
from pathlib import Path


def u2(data, at):
    return struct.unpack_from(">H", data, at)[0]


def u4(data, at):
    return struct.unpack_from(">I", data, at)[0]


def put_u2(data, at, value):
    struct.pack_into(">H", data, at, value)


def parse_class(data):
    if data[:4] != b"\xca\xfe\xba\xbe":
        raise ValueError("not a class file")
    cp_count = u2(data, 8)
    cp, offset, index = [None] * cp_count, 10, 1
    while index < cp_count:
        tag = data[offset]
        offset += 1
        if tag == 1:
            length = u2(data, offset)
            offset += 2
            cp[index] = (tag, data[offset:offset + length].decode("utf-8"))
            offset += length
        elif tag in (3, 4):
            cp[index] = (tag,)
            offset += 4
        elif tag in (5, 6):
            cp[index] = (tag,)
            offset += 8
            index += 1
        elif tag in (7, 8, 16, 19, 20):
            cp[index] = (tag, u2(data, offset))
            offset += 2
        elif tag in (9, 10, 11, 12, 17, 18):
            cp[index] = (tag, u2(data, offset), u2(data, offset + 2))
            offset += 4
        elif tag == 15:
            cp[index] = (tag, data[offset], u2(data, offset + 1))
            offset += 3
        else:
            raise ValueError(f"unsupported constant-pool tag {tag} at #{index}")
        index += 1

    def utf8(i):
        return cp[i][1]

    def class_name(i):
        return utf8(cp[i][1])

    this_class = u2(data, offset + 2)
    offset += 6
    interfaces = u2(data, offset)
    offset += 2 + 2 * interfaces

    def skip_members(pos, count):
        for _ in range(count):
            attrs = u2(data, pos + 6)
            pos += 8
            for _ in range(attrs):
                length = u4(data, pos + 2)
                pos += 6 + length
        return pos

    fields = u2(data, offset)
    offset = skip_members(offset + 2, fields)
    methods_count = u2(data, offset)
    offset += 2
    test_code = None
    for _ in range(methods_count):
        name, desc, attrs = u2(data, offset + 2), u2(data, offset + 4), u2(data, offset + 6)
        offset += 8
        for _ in range(attrs):
            attr_name, length = u2(data, offset), u4(data, offset + 2)
            body = offset + 6
            if utf8(attr_name) == "Code" and utf8(name) == "test" and utf8(desc) == "(I)V":
                code_length = u4(data, body + 4)
                code_at = body + 8
                table_len_at = code_at + code_length
                table_count = u2(data, table_len_at)
                table_at = table_len_at + 2
                test_code = (code_at, code_length, table_at, table_count)
            offset += 6 + length
    if test_code is None:
        raise ValueError("test(I)V Code attribute not found")

    refs = {}
    for i, item in enumerate(cp):
        if item and item[0] == 10:
            owner_index, nt_index = item[1], item[2]
            nt = cp[nt_index]
            refs[(class_name(owner_index), utf8(nt[1]), utf8(nt[2]))] = i
    return cp, refs, test_code, class_name(this_class)


def main():
    if len(sys.argv) != 4:
        raise SystemExit("usage: mutate_class.py INPUT.class OUTPUT.class MUTATION")
    src, dst, mutation = Path(sys.argv[1]), Path(sys.argv[2]), sys.argv[3]
    data = bytearray(src.read_bytes())
    cp, refs, (code_at, code_length, table_at, table_count), owner = parse_class(data)
    before = data[:]
    if mutation == "cleanup-target":
        bci = 11
        if code_length <= bci + 2 or data[code_at + bci] != 0xB6:
            raise ValueError("expected invokevirtual at BCI 11")
        old_index = u2(data, code_at + bci + 1)
        new_index = refs[(owner, "doSomething3", "()V")]
        put_u2(data, code_at + bci + 1, new_index)
        detail = f"test(I)V BCI 11 Methodref #{old_index} doSomething4()V -> #{new_index} doSomething3()V"
    elif mutation == "range-expanded":
        rows = []
        for index in range(table_count):
            at = table_at + index * 8
            row = (u2(data, at), u2(data, at + 2), u2(data, at + 4), u2(data, at + 6))
            rows.append(row)
        match = next((i for i, row in enumerate(rows) if row == (0, 10, 56, 0)), None)
        if match is None:
            raise ValueError(f"catch-all [0,10)->56 not found: {rows}")
        put_u2(data, table_at + match * 8 + 2, 14)
        detail = f"exception table row {match}: catch-all [0,10)->56 -> [0,14)->56; newly covers early cleanup BCIs 10..13"
    elif mutation == "branch-bypass":
        bci = 7
        if code_length <= bci + 2 or data[code_at + bci] != 0xA0:
            raise ValueError("expected if_icmpne at BCI 7")
        old = struct.unpack_from(">h", data, code_at + bci + 1)[0]
        old_target = bci + old
        new_offset = 63 - bci
        struct.pack_into(">h", data, code_at + bci + 1, new_offset)
        detail = f"test(I)V BCI 7 if_icmpne target {old_target} -> 63 (return), bypassing branch work and normal cleanup"
    elif mutation == "rethrow-changed":
        bci = 61
        if code_length <= bci + 1 or data[code_at + bci] != 0x2D or data[code_at + bci + 1] != 0xBF:
            raise ValueError("expected aload_3; athrow at BCI 61..62")
        data[code_at + bci] = 0x01  # aconst_null; verifier permits null as Throwable input to athrow
        detail = "test(I)V BCI 61 aload_3 (original Throwable) -> aconst_null; BCI 62 athrow now completes with NullPointerException"
    else:
        raise ValueError(f"unknown mutation: {mutation}")
    dst.parent.mkdir(parents=True, exist_ok=True)
    dst.write_bytes(data)
    print(f"mutation={mutation}")
    print(f"class={owner}")
    print(detail)
    print(f"class_bytes_changed={sum(a != b for a, b in zip(before, data))}")


if __name__ == "__main__":
    main()
