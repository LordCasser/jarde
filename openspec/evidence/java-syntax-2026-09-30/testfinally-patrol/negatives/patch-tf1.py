#!/usr/bin/env python3
"""Re-apply the four same-length bytecode patches that turn verifier-valid Tf1-shaped
classes into negative neighbors the local-null certificate must refuse.

Each base class is the fixed Tf1 lowering itself (recompiled from `src/<Name>/` first), and
each patch edits one copy or one completion in place, so every class stays verifier-valid
(`java -Xverify:all` passes) and StackMapTable-compatible:

* Tf1InvertedCondition — the normal copy's `ifnull` becomes `ifnonnull`, so the branch's
  null direction is inverted (the cleanup then runs on the null path).
* Tf1TargetMismatch    — the normal copy's `invokevirtual Cursor.close()V` index moves to
  `Cursor.moveToFirst()V`, so the two copies call different targets.
* Tf1ReturnIdentity    — the normal completion's `aload 6` (the saved return) becomes
  `aconst_null; nop`, so the saved-return identity is rewritten.
* Tf1RethrowIdentity   — the handler completion's `aload 7` (the pending throwable) becomes
  `aconst_null; nop`, so the rethrow identity is rewritten.

Usage: python3 patch-tf1.py   (from this directory; recompiles the sources first)
"""
import re
import struct
import subprocess
from pathlib import Path

HERE = Path(__file__).parent
SRC = HERE / "src"
BASES = ["Tf1InvertedCondition", "Tf1TargetMismatch", "Tf1ReturnIdentity", "Tf1RethrowIdentity"]

# One cleanup copy's own bytes: `aload_3; ifnull +n; aload_3; invokevirtual #i`. The fixed
# lowering has exactly two (normal first, handler second); the patches key on that order.
COPY = re.compile(rb'\x2d\xc6(.)(.)\x2d\xb6', re.S)


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


def copies(code):
    found = list(COPY.finditer(code))
    if len(found) != 2:
        raise SystemExit(f"expected two cleanup copies, found {len(found)}")
    return found


def compile_and_read(name):
    out = SRC / name
    subprocess.run(
        ["javac", "--release", "8", "-g:none", "-Xlint:-options", "-d", str(out),
         str(SRC / f"{name}.java")],
        check=True, capture_output=True)
    return bytearray((out / f"{name}.class").read_bytes())


def write(name, data):
    (SRC / name / f"{name}.class").write_bytes(bytes(data))


def main():
    for name in BASES:
        data = compile_and_read(name)
        utf8, classes, nat, refs = pool(data)
        # The `test` method's code is the only place the copy pattern occurs; every patch
        # edits inside it.
        found = copies(data)
        if name == "Tf1InvertedCondition":
            normal = found[0]
            i = normal.start()
            if data[i + 1] != 0xC6:
                raise SystemExit("normal copy's branch is not ifnull")
            data[i + 1] = 0xC7
        elif name == "Tf1TargetMismatch":
            normal = found[0]
            invoke_at = normal.start() + 5
            target = methodref(utf8, classes, nat, refs, "Cursor", "moveToFirst", "()V")
            data[invoke_at + 1:invoke_at + 3] = struct.pack('>H', target)
        elif name == "Tf1ReturnIdentity":
            aload, ret = b'\x19\x06', b'\xb0'
            at = data.find(aload + ret)
            if at < 0 or data.find(aload + ret, at + 1) >= 0:
                raise SystemExit("saved-return load is not unique")
            data[at:at + 2] = b'\x01\x00'
        elif name == "Tf1RethrowIdentity":
            aload, throw = b'\x19\x07', b'\xbf'
            at = data.find(aload + throw)
            if at < 0 or data.find(aload + throw, at + 1) >= 0:
                raise SystemExit("pending-throwable load is not unique")
            data[at:at + 2] = b'\x01\x00'
        write(name, data)
        print(f"{name}: patched")


if __name__ == "__main__":
    main()
