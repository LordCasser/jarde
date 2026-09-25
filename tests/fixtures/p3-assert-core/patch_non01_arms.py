#!/usr/bin/env python3
"""Change only the two <clinit> stack-Phi inputs written to the Z field."""

import argparse
import hashlib
from pathlib import Path


ORIGINAL_SHA256 = "71753fbec0b66a6511bbb2b66855c50589060b729bc963cb6cd78377b75f4aaf"
PATCHED_SHA256 = "b52d39dd6ae7943d70b510c4925f23faadcc2fcc64c5cb7a2c309ae450de4e2c"
# <clinit> BCI 8: iconst_1; goto 13; BCI 12: iconst_0; BCI 13: putstatic #18 Z.
ORIGINAL = bytes.fromhex("04 a7 00 04 03 b3 00 12")
PATCHED = bytes.fromhex("05 a7 00 04 06 b3 00 12")


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
