#!/usr/bin/env python3
"""Change only `<clinit>`'s assertion-status class literal in the frozen class."""

import argparse
import hashlib
from pathlib import Path


ORIGINAL_SHA256 = "7933b18c609d21d549b1c2c5b9356ad4ebd40a811b48cbbe06af3af66437dfde"
PATCHED_SHA256 = "539af1f7fba5b8e543ff67048707c2c79f0ca570dfce5d168449dca8ce3e3232"
# `ldc #8` (AssertProbe.class), `invokevirtual #74` (desiredAssertionStatus).
# Constant pool #16 is java/lang/StringBuilder; both operands are u1 class indexes.
ORIGINAL = bytes.fromhex("12 08 b6 00 4a")
PATCHED = bytes.fromhex("12 10 b6 00 4a")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    source = args.input.read_bytes()
    assert hashlib.sha256(source).hexdigest() == ORIGINAL_SHA256
    assert source.count(ORIGINAL) == 1
    result = source.replace(ORIGINAL, PATCHED, 1)
    assert hashlib.sha256(result).hexdigest() == PATCHED_SHA256
    args.output.write_bytes(result)


if __name__ == "__main__":
    main()
