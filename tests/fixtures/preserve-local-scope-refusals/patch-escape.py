#!/usr/bin/env python3
"""Derive the nested/shared-handler escape control from the committed fixture class.

The fixture's `sharedHandler` binds the caught reference in slot 2 (`astore_2`), reads it in the
clause's own body and returns the method local of slot 1. This script rewrites the unique five-byte
sequence `astore_2; aload_2; astore_1; aload_1; areturn` (`4d 2c 4c 2b b0`) so the handler's entry
store and load address slot 1 instead (`4c 2b 4c 2b b0`): the caught value is then written into the
method local's slot and read by the return after the clause — verifier-valid bytecode whose binding
no Java lexical scope can spell.

Usage: patch-escape.py <input class> <output class>

The script self-tests before it writes: the needle must occur exactly once in the input, and the
patch must change exactly those five bytes. It prints both digests; the fixture README records the
expected ones and the `-Xverify:all` replay.
"""

import hashlib
import sys
from pathlib import Path

NEEDLE = bytes([0x4D, 0x2C, 0x4C, 0x2B, 0xB0])
REPLACEMENT = bytes([0x4C, 0x2B, 0x4C, 0x2B, 0xB0])


def main() -> int:
    if len(sys.argv) != 3:
        print(__doc__, file=sys.stderr)
        return 2
    source = Path(sys.argv[1])
    target = Path(sys.argv[2])
    data = source.read_bytes()
    sites = data.count(NEEDLE)
    if sites != 1:
        print(f"self-test failed: the needle occurs {sites} times, not once", file=sys.stderr)
        return 1
    patched = data.replace(NEEDLE, REPLACEMENT)
    if len(patched) != len(data) or patched == data:
        print("self-test failed: the patch did not keep every BCI", file=sys.stderr)
        return 1
    target.write_bytes(patched)
    print(f"{source}: {len(data)} bytes sha256 {hashlib.sha256(data).hexdigest()}")
    print(f"{target}: {len(patched)} bytes sha256 {hashlib.sha256(patched).hexdigest()}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
