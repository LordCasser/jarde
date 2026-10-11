#!/usr/bin/env python3
"""Patch the frozen CompoundStatementConditions test method into a JSR clone-path fixture.

This only rewrites one method's Code attribute and the class major version. It does not
invoke javac, a JVM, Cargo, or the project CLI.
"""
from pathlib import Path
import hashlib, json, struct, sys

EXPECTED_2fc584d5ba9d35940516fcf9001479a72851ebb0216c22a62f3ec5500dbc35aa = "2fc584d5ba9d35940516fcf9001479a72851ebb0216c22a62f3ec5500dbc35aa"
METHOD_NAME = b"test"
METHOD_DESCRIPTOR = b"(ZZZZLjava/lang/StringBuilder;)Ljava/lang/String;"
# caller: jsr 8; jsr 8; aconst_null; areturn
# subroutine: astore 5; (a && b) || (c && d); one shared ret 5 after both outcome blocks
CODE = bytes.fromhex(
    "a80008 a80005 01b0 3a05 "
    "1a 99000a "
    "1b 990011 "
    "a7000b "
    "1c 99000a "
    "1d 990006 "
    "a70006 a70003 a905"
)


def u1(data, at): return data[at], at + 1
def u2(data, at): return struct.unpack_from(">H", data, at)[0], at + 2
def u4(data, at): return struct.unpack_from(">I", data, at)[0], at + 4


def parse_cp(data):
    count = struct.unpack_from(">H", data, 8)[0]
    at = 10
    cp = [None] * count
    index = 1
    while index < count:
        tag, at = u1(data, at)
        if tag == 1:
            length, at = u2(data, at)
            cp[index] = data[at:at + length]
            at += length
        elif tag in (3, 4): at += 4
        elif tag in (5, 6):
            at += 8
            index += 1
        elif tag in (7, 8, 16, 19, 20): at += 2
        elif tag in (9, 10, 11, 12, 17, 18): at += 4
        elif tag == 15: at += 3
        else: raise ValueError(f"unsupported constant-pool tag {tag} at index {index}")
        index += 1
    return cp, at


def read_member(data, at, cp):
    access, at = u2(data, at)
    name_i, at = u2(data, at)
    desc_i, at = u2(data, at)
    count, at = u2(data, at)
    attrs = []
    for _ in range(count):
        attr_name_i, at = u2(data, at)
        length, at = u4(data, at)
        info = data[at:at + length]
        at += length
        attrs.append((attr_name_i, info))
    return (access, cp[name_i], cp[desc_i], attrs), at


def patch(data):
    if hashlib.sha256(data).hexdigest() != EXPECTED_2fc584d5ba9d35940516fcf9001479a72851ebb0216c22a62f3ec5500dbc35aa:
        raise ValueError("controlled source class SHA-256 mismatch")
    if data[:4] != b"\xca\xfe\xba\xbe":
        raise ValueError("not a classfile")
    minor = struct.unpack_from(">H", data, 4)[0]
    major = struct.unpack_from(">H", data, 6)[0]
    cp, at = parse_cp(data)
    # access_flags, this_class, super_class
    at += 6
    interfaces, at = u2(data, at)
    at += 2 * interfaces
    fields, at = u2(data, at)
    for _ in range(fields):
        _, at = read_member(data, at, cp)
    methods_count_at = at
    methods_count, at = u2(data, at)
    methods_start = at
    output_methods = []
    found = 0
    for _ in range(methods_count):
        start = at
        member, at = read_member(data, at, cp)
        access, name, descriptor, attrs = member
        if name == METHOD_NAME and descriptor == METHOD_DESCRIPTOR:
            found += 1
            code_attrs = [i for i, (name_index, _) in enumerate(attrs)
                          if cp[name_index] == b"Code"]
            if len(code_attrs) != 1:
                raise ValueError(f"expected one Code attribute, got {len(code_attrs)}")
            code_info = struct.pack(">HHI", 1, 6, len(CODE)) + CODE + struct.pack(">HH", 0, 0)
            attrs[code_attrs[0]] = (attrs[code_attrs[0]][0], code_info)
        # Keep the original name/descriptor indexes from the method header.
        old = data[start:at]
        old_name_i = struct.unpack_from(">H", old, 2)[0]
        old_desc_i = struct.unpack_from(">H", old, 4)[0]
        method = bytearray(struct.pack(">HHHH", access, old_name_i, old_desc_i, len(attrs)))
        for attr_name_i, info in attrs:
            method += struct.pack(">HI", attr_name_i, len(info)) + info
        output_methods.append(bytes(method))
    if found != 1:
        raise ValueError(f"expected one target method, found {found}")
    methods_end = at
    # Preserve class attributes byte-for-byte after the replaced method table.
    patched = bytearray(data[:6] + struct.pack(">H", 49))
    patched += data[8:methods_count_at]
    patched += struct.pack(">H", methods_count) + b"".join(output_methods)
    patched += data[methods_end:]
    return bytes(patched), {"source_minor": minor, "source_major": major, "target_major": 49,
                            "method": METHOD_NAME.decode(), "descriptor": METHOD_DESCRIPTOR.decode(),
                            "max_stack": 1, "max_locals": 6, "code_length": len(CODE),
                            "code_hex": CODE.hex(), "subroutine_entry_bci": 8,
                            "jsr_call_sites": [0, 3], "caller_join_bci": 3,
                            "comparison_branch_bcis": [11, 15, 22, 26],
                            "ret_bcis": [35], "code_exception_table_entries": 0,
                            "code_nested_attributes": 0}


def main():
    source = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(__file__).with_name("original.class")
    output = Path(sys.argv[2]) if len(sys.argv) > 2 else Path(__file__).with_name("CompoundStatementConditions.class")
    info_path = Path(__file__).with_name("fixture.json")
    original = source.read_bytes()
    patched, info = patch(original)
    if output.exists() or info_path.exists():
        raise FileExistsError("refusing to overwrite an existing private fixture output")
    output.write_bytes(patched)
    info.update(source_path=str(source), source_sha256=hashlib.sha256(original).hexdigest(),
                output_path=str(output), output_sha256=hashlib.sha256(patched).hexdigest())
    info_path.write_text(json.dumps(info, indent=2) + "\n")
    print(json.dumps(info, sort_keys=True))


if __name__ == "__main__":
    main()
