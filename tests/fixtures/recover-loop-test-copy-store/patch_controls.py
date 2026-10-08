#!/usr/bin/env python3
"""Make the multi-consumer control of this fixture from the anchor's own compiled class.

`Probe.readAll`'s loop test is `aload_1; invokevirtual FileReader.read:()I; dup; istore_3;
iconst_m1; if_icmpeq`, and its body begins `aload_2; iload_3; i2c; invokevirtual append(C)`. The
patch replaces the `iconst_m1` at the test's second operand position with a `dup` (the same one
byte): the surviving copy is consumed by that copy rather than by the comparison, so the copy the
store takes has a second consumer and the test reads a copy of a copy. JVM integer stack
verification still passes — the comparison compares two ints — and the class loads and runs under
`java -Xverify:all`.

The site is found by the loop test's first four instructions (`dup; istore_3; iconst_m1;
if_icmpeq`) followed by the body's first three (`aload_2; iload_3; i2c`): it names `readAll` alone
(the class's other loop, `guardPlain`'s, adds `iload_2; iload_3; iadd; istore_2` instead), and the
branch's own offset bytes between them are read, not matched, so the two legs patch alike.

    patch_controls.py <leg>/Probe.class <leg>/MultiCopy.class
"""

from pathlib import Path
import sys


original = Path(sys.argv[1]).read_bytes()
output = Path(sys.argv[2])

head = bytes.fromhex("59 3e 02 9f")  # dup; istore_3; iconst_m1; if_icmpeq
tail = bytes.fromhex("2c 1d 92")  # aload_2; iload_3; i2c — readAll's body, not guardPlain's
sites = []
index = original.find(head)
while index >= 0:
    if original[index + 6 : index + 9] == tail:
        sites.append(index)
    index = original.find(head, index + 1)
assert len(sites) == 1, "the readAll loop test is the committed shape"

# `iconst_m1` (0x02) becomes `dup` (0x59): the same width, so the offsets after it do not move.
index = sites[0]
patched = original[: index + 2] + bytes.fromhex("59") + original[index + 3 :]
output.write_bytes(patched)
print(f"{output} written")
