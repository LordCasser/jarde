#!/usr/bin/env python3
"""Turn the two proved 0/1 ternary producers into verifier-valid 2/3 producers."""

from pathlib import Path

root = Path(__file__).resolve().parent
source = root / "ConditionalFieldWrites.class"
target = root / "ConditionalFieldWritesNon01.class"
data = source.read_bytes()
for before, after in (
    (
        bytes.fromhex("1a 99 00 07 04 a7 00 04 03 b3"),
        bytes.fromhex("1a 99 00 07 05 a7 00 04 06 b3"),
    ),
    (
        bytes.fromhex("2a 1b 99 00 07 04 a7 00 04 03 b5"),
        bytes.fromhex("2a 1b 99 00 07 05 a7 00 04 06 b5"),
    ),
):
    if data.count(before) != 1:
        raise SystemExit(f"expected one Code sequence {before.hex()}; found {data.count(before)}")
    data = data.replace(before, after)
target.write_bytes(data)
