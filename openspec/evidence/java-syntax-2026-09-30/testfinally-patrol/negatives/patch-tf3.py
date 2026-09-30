#!/usr/bin/env python3
"""Re-apply the seven same-length bytecode patches that turn verifier-valid Tf3-shaped
classes into negative neighbors the segmented null-lead finally certificate must refuse.

Each base class is the fixed Tf3 lowering itself (recompiled from `src/<Name>.java` first),
and each patch edits one copy, one completion, one condition, or one exception-table row in
place, so every class stays verifier-valid (`java -Xverify:all` passes) and
StackMapTable-compatible:

* Tf3TargetMismatch   — the handler copy's `invokestatic close` index moves to the unused
  same-descriptor `closeOther`, so the three copies no longer share one target.
* Tf3ArgOtherSlot     — the early copy's `aload_1` (the lead slot) becomes `aload_2` (the
  saved value's own slot), so that copy's argument no longer reads the lead slot.
* Tf3ReturnIdentity   — the early completion's `aload_2` (the saved `null`) becomes
  `aconst_null`, so the return no longer reads the value slot its own store saved.
* Tf3SelfRowWidened   — the second protected row widens from [24,47) to [24,55), so the table
  covers the handler's own binding store and its cleanup call: a self-protection row exists.
* Tf3CondNonNullRewrite — the body test's `ifnonnull 38` becomes `ifnull 38`, so the branch's
  sense no longer proves the `if (bytes == null)` fall-through shape.
* Tf3CondIfneRewrite  — the inner guard's `ifne 24` becomes `ifeq 24`, so the branch's sense
  no longer proves the `if (!validate())` guard shape.
* Tf3RethrowIdentity  — the handler completion's `aload_3` (the pending throwable) becomes
  `aconst_null`, so the rethrow identity is rewritten (null throws a verifier-valid NPE at
  run time).

Usage: python3 patch-tf3.py   (from this directory; recompiles the sources first)
"""
import struct
import subprocess
from pathlib import Path

HERE = Path(__file__).parent
SRC = HERE / "src"
OPCODE_BASES = [
    "Tf3TargetMismatch",
    "Tf3ArgOtherSlot",
    "Tf3ReturnIdentity",
    "Tf3CondNonNullRewrite",
    "Tf3CondIfneRewrite",
    "Tf3RethrowIdentity",
]
TABLE_BASES = ["Tf3SelfRowWidened"]


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


def test_code(data, method_name=b"test", descriptor=b"()[B"):
    """The `Code` attribute bytes of `<method><descriptor>`, as (start, end) into data."""
    utf8, _, _, _ = pool(data)
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
    _, _, _ = struct.unpack('>HHH', data[i:i + 6])
    i += 6
    interfaces = struct.unpack('>H', data[i:i + 2])[0]
    i += 2 + 2 * interfaces
    fields_count = struct.unpack('>H', data[i:i + 2])[0]
    i += 2

    def skip_members(i, member_count):
        for _ in range(member_count):
            _, name_index, descriptor_index = struct.unpack('>HHH', data[i:i + 6])
            i += 6
            attributes = struct.unpack('>H', data[i:i + 2])[0]
            i += 2
            for _ in range(attributes):
                _, length = struct.unpack('>HI', data[i:i + 6])
                i += 6 + length
        return i

    i = skip_members(i, fields_count)
    methods_count = struct.unpack('>H', data[i:i + 2])[0]
    i += 2
    for _ in range(methods_count):
        _, name_index, descriptor_index = struct.unpack('>HHH', data[i:i + 6])
        attributes = struct.unpack('>H', data[i + 6:i + 8])[0]
        j = i + 8
        for _ in range(attributes):
            attr_name, length = struct.unpack('>HI', data[j:j + 6])
            if attr_name == utf8_index(utf8, b"Code") and (
                utf8[name_index], utf8[descriptor_index]
            ) == (method_name, descriptor):
                return j + 6, j + 6 + length
            j += 6 + length
        i = j
    raise SystemExit(f"no Code attribute for {method_name}{descriptor}")


def utf8_index(utf8, raw):
    for index, value in utf8.items():
        if value == raw:
            return index
    raise SystemExit(f"no utf8 {raw}")


def patch_opcodes(name, edits):
    """edits: list of (code_offset_within_test, expected_byte, new_byte)."""
    data = compile_and_read(name)
    start, end = test_code(data)
    for offset, expected, new in edits:
        # The code bytes start after the attribute's own `max_stack`/`max_locals`/`code_length`
        # header: eight bytes into the `Code` attribute.
        at = start + 8 + offset
        if data[at] != expected:
            raise SystemExit(f"{name}: byte at code offset {offset} is {data[at]:#x}, want {expected:#x}")
        data[at] = new
    write(name, data)
    print(f"patched {name}")


def patch_row_end(name, row_index, new_end):
    data = compile_and_read(name)
    start, end = test_code(data)
    utf8, classes, nat, refs = pool(data)
    # The Code attribute: [max_stack u2][max_locals u2][code_length u4][code..]
    # [exception_table_length u2][entries of 8 bytes..][attributes..]
    code_length = struct.unpack('>I', data[start + 4:start + 8])[0]
    table_at = start + 8 + code_length
    entries = struct.unpack('>H', data[table_at:table_at + 2])[0]
    if entries != 2:
        raise SystemExit(f"{name}: expected the fixed two-row table, found {entries}")
    at = table_at + 2 + 8 * row_index + 2  # end_pc of the row
    old_end = struct.unpack('>H', data[at:at + 2])[0]
    data[at:at + 2] = struct.pack('>H', new_end)
    print(f"patched {name}: row {row_index} end_pc {old_end} -> {new_end}")
    write(name, data)


def patch_handler_target(name):
    data = compile_and_read(name)
    start, end = test_code(data)
    utf8, classes, nat, refs = pool(data)
    owner = name
    close = methodref(utf8, classes, nat, refs, owner, "close", "(Ljava/io/InputStream;)V")
    close_other = methodref(
        utf8, classes, nat, refs, owner, "closeOther", "(Ljava/io/InputStream;)V")
    if close_other == close:
        raise SystemExit(f"{name}: close and closeOther share one methodref")
    # The handler copy's `invokestatic` is the instruction at BCI 55 (0x37 into the code):
    # [u2 max_stack][u2 max_locals][u4 code_length] puts the code at start+8; BCI 55 follows.
    at = start + 8 + 55 + 1
    if struct.unpack('>H', data[at:at + 2])[0] != close:
        raise SystemExit(f"{name}: handler copy does not call close")
    data[at:at + 2] = struct.pack('>H', close_other)
    write(name, data)
    print(f"patched {name}: handler copy -> closeOther")


def main():
    patch_handler_target("Tf3TargetMismatch")
    # The early copy's argument: 18 is `aload_1` (0x2b) in the fixed lowering; slot 2 is 0x2c.
    patch_opcodes("Tf3ArgOtherSlot", [(18, 0x2B, 0x2C)])
    # The early completion's read: 22 is `aload_2` (0x2c); a fresh `aconst_null` is 0x01.
    patch_opcodes("Tf3ReturnIdentity", [(22, 0x2C, 0x01)])
    # The second row widens over the handler's binding store and its own copy.
    patch_row_end("Tf3SelfRowWidened", 1, 55)
    # The body test's sense: 6 is `ifnonnull` (0xc7); `ifnull` is 0xc6.
    patch_opcodes("Tf3CondNonNullRewrite", [(6, 0xC7, 0xC6)])
    # The inner guard's sense: 13 is `ifne` (0x9a); `ifeq` is 0x99.
    patch_opcodes("Tf3CondIfneRewrite", [(13, 0x9A, 0x99)])
    # The handler's rethrow read: 58 is `aload_3` (0x2d); a fresh `aconst_null` is 0x01.
    patch_opcodes("Tf3RethrowIdentity", [(58, 0x2D, 0x01)])


if __name__ == "__main__":
    main()
