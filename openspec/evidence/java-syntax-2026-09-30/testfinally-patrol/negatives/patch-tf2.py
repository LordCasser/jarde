#!/usr/bin/env python3
"""Re-apply the four same-length bytecode patches that turn verifier-valid Tf2-shaped
classes into negative neighbors the null-lead straight-finally certificate must refuse.

Each base class is the fixed Tf2 lowering itself (recompiled from `src/<Name>.java` first),
and each patch edits one copy, one completion, or one exception-table row in place, so every
class stays verifier-valid (`java -Xverify:all` passes) and StackMapTable-compatible:

* Tf2TargetMismatch   — the handler copy's `invokespecial closeQuietly` index moves to the
  unused same-descriptor `closeOther`, so the two copies call different targets.
* Tf2ReturnIdentity   — the normal completion's `aload_3` (the saved return) becomes
  `aconst_null`, so the return hands back null instead of the constructed value.
* Tf2RethrowIdentity  — the handler completion's wide `aload 4` (the pending throwable)
  becomes `aconst_null; nop`, so the rethrow identity is rewritten (null throws a
  verifier-valid NPE at run time).
* Tf2SelfRowWidened   — the handler's own binding row widens from [32,34) to [32,41), so the
  row covers the handler cleanup's own call.

Usage: python3 patch-tf2.py   (from this directory; recompiles the sources first)
"""
import re
import struct
import subprocess
from pathlib import Path

HERE = Path(__file__).parent
SRC = HERE / "src"
CODE_BASES = ["Tf2TargetMismatch", "Tf2ReturnIdentity", "Tf2RethrowIdentity"]
TABLE_BASES = ["Tf2SelfRowWidened"]


def pool(data):
    """Parse the constant pool enough to resolve NameAndType/Class/Memberref indexes."""
    count = struct.unpack('>H', data[8:10])[0]
    i, idx = 10, 1
    utf8, classes, nat, refs = {}, {}, {}, {}
    while idx < count:
        tag = data[i]
        if tag == 1:
            length = struct.unpack('>H', data[i + 1:i + 3])[0]
            utf8[idx] = data[i + 3:i + 3 + length]
            i += 3 + length
        elif tag == 7:
            classes[idx] = struct.unpack('>H', data[i + 1:i + 3])[0]
            i += 3
        elif tag in (9, 10, 11):
            refs[idx] = struct.unpack('>HH', data[i + 1:i + 5])
            i += 5
        elif tag == 12:
            nat[idx] = struct.unpack('>HH', data[i + 1:i + 5])
            i += 5
        elif tag in (3, 4):
            i += 5
        elif tag in (5, 6):
            i += 9
        elif tag == 8:
            i += 3
        else:
            raise SystemExit(f"unexpected constant-pool tag {tag} at {i}")
        idx += 1
    return utf8, classes, nat, refs


def methodref(utf8, classes, nat, refs, owner, name, descriptor):
    owner, name, descriptor = owner.encode(), name.encode(), descriptor.encode()
    for index, (class_index, nat_index) in refs.items():
        if utf8[classes[class_index]] != owner:
            continue
        n, t = nat[nat_index]
        if utf8[n] == name and utf8[t] == descriptor:
            return index
    raise SystemExit(f"no {owner}.{name}{descriptor} methodref")


def compile_and_read(name):
    out = SRC / name
    subprocess.run(
        ["javac", "--release", "8", "-g:none", "-Xlint:-options", "-d", str(out),
         str(SRC / f"{name}.java")],
        check=True, capture_output=True)
    return bytearray((out / f"{name}.class").read_bytes())


def write(name, data):
    (SRC / name / f"{name}.class").write_bytes(bytes(data))


def test_code(data):
    """The `Code` attribute bytes of `test([B)LTf2$Result;`, as (start, end) into data."""
    utf8 = pool(data)[0]
    count = struct.unpack('>H', data[8:10])[0]
    # Re-walk the pool to find where it ends.
    i, idx = 10, 1
    while idx < count:
        tag = data[i]
        if tag == 1:
            i += 3 + struct.unpack('>H', data[i + 1:i + 3])[0]
        elif tag in (7, 8, 16, 19, 20):
            i += 3
        elif tag in (9, 10, 11, 12, 17, 18):
            i += 5
        elif tag in (3, 4):
            i += 5
        elif tag in (5, 6):
            i += 9
        else:
            raise SystemExit(f"unexpected tag {tag}")
        idx += 1
    access_flags, this_class, super_class = struct.unpack('>HHH', data[i:i + 6])
    i += 6
    interfaces = struct.unpack('>H', data[i:i + 2])[0]
    i += 2 + 2 * interfaces
    fields_count = struct.unpack('>H', data[i:i + 2])[0]
    i += 2

    def skip_members(i, member_count):
        for _ in range(member_count):
            flags, name_index, descriptor_index = struct.unpack('>HHH', data[i:i + 6])
            i += 6
            attributes = struct.unpack('>H', data[i:i + 2])[0]
            i += 2
            for _ in range(attributes):
                name_index, length = struct.unpack('>HI', data[i:i + 6])
                i += 6 + length
        return i

    i = skip_members(i, fields_count)
    methods_count = struct.unpack('>H', data[i:i + 2])[0]
    i += 2
    for _ in range(methods_count):
        _flags, name_index, descriptor_index = struct.unpack('>HHH', data[i:i + 6])
        i += 6
        attributes = struct.unpack('>H', data[i:i + 2])[0]
        i += 2
        for _ in range(attributes):
            attribute_name, length = struct.unpack('>HI', data[i:i + 6])
            body = i + 6
            if (utf8[attribute_name] == b"Code" and utf8[name_index] == b"test"
                    and utf8[descriptor_index].startswith(b"([B)LTf2")):
                return body, body + length
            i = body + length
    raise SystemExit("no test([B)...Result; Code attribute")


def find_in_test(data, pattern):
    """The absolute offsets of one byte pattern inside the `test` method's own code."""
    start, end = test_code(data)
    code_start = start + 8
    return [code_start + m.start()
            for m in re.finditer(pattern, bytes(data[code_start:end]))]


def main():
    for name in CODE_BASES:
        data = compile_and_read(name)
        utf8, classes, nat, refs = pool(data)
        if name == "Tf2TargetMismatch":
            cleanup = methodref(utf8, classes, nat, refs, "Tf2TargetMismatch",
                                "closeQuietly", "(Ljava/io/InputStream;)V")
            copies = find_in_test(
                data, rb'\x2a\x2c\xb7' + struct.pack('>H', cleanup))
            if len(copies) != 2:
                raise SystemExit(f"expected two call copies, found {len(copies)}")
            invoke_at = copies[1] + 2
            target = methodref(utf8, classes, nat, refs, "Tf2TargetMismatch",
                               "closeOther", "(Ljava/io/InputStream;)V")
            data[invoke_at + 1:invoke_at + 3] = struct.pack('>H', target)
        elif name == "Tf2ReturnIdentity":
            pair = rb'\x2d\xb0'
            found = find_in_test(data, pair)
            if len(found) != 1:
                raise SystemExit("saved-return load is not unique")
            data[found[0]] = 0x01
        elif name == "Tf2RethrowIdentity":
            pair = rb'\x19\x04\xbf'
            found = find_in_test(data, pair)
            if len(found) != 1:
                raise SystemExit("pending-throwable load is not unique")
            data[found[0]:found[0] + 3] = b'\x01\x00\xbf'
        write(name, data)
        print(f"{name}: patched")
    for name in TABLE_BASES:
        data = compile_and_read(name)
        start, end = test_code(data)
        # Code attribute: max_stack u2, max_locals u2, code_length u4, code, then the table.
        code_length = struct.unpack('>I', data[start + 4:start + 8])[0]
        table_length = struct.unpack('>H', data[start + 8 + code_length:
                                                start + 10 + code_length])[0]
        entries = start + 10 + code_length
        if table_length != 2:
            raise SystemExit(f"expected two exception rows, found {table_length}")
        rows = [struct.unpack('>HHHH', data[entries + 8 * k:entries + 8 * k + 8])
                for k in range(table_length)]
        if rows[1] != (32, 34, 32, 0):
            raise SystemExit(f"the self-protecting row is not [32,34)->32: {rows[1]}")
        data[entries + 8 + 2:entries + 8 + 4] = struct.pack('>H', 41)
        write(name, data)
        print(f"{name}: patched")


if __name__ == "__main__":
    main()
