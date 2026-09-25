#!/usr/bin/env python3
"""Retarget only the assertion-status class literal in the frozen Java 8 class."""

import argparse
import hashlib
from pathlib import Path


ORIGINAL_SHA256 = "71753fbec0b66a6511bbb2b66855c50589060b729bc963cb6cd78377b75f4aaf"
PATCHED_SHA256 = "af3c1190da44c938138da327a6232220acd3b8e8161808e2d9d517f11a4e71dc"
# `<clinit>`: ldc #8 (AssertCore.class); invokevirtual #50 (desiredAssertionStatus).
# Class constant #35 is java/lang/StringBuilder. Both class entries already exist.
ORIGINAL = bytes.fromhex("12 08 b6 00 32")
PATCHED = bytes.fromhex("12 23 b6 00 32")


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
