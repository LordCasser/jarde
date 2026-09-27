#!/usr/bin/env python3
"""Redirect the normal-cleanup transfer into the second protected segment."""
import struct
import sys
from pathlib import Path

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "negatives"))
from mutate_class import parse_class  # noqa: E402

source, output = map(Path, sys.argv[1:])
data = bytearray(source.read_bytes())
_, _, (code_at, code_length, _, _), _ = parse_class(data)
if code_length <= 43 or data[code_at + 41] != 0xA7:
    raise SystemExit("expected goto at test(I)V BCI 41")
if struct.unpack_from(">h", data, code_at + 42)[0] != 63 - 41:
    raise SystemExit("unexpected original normal-cleanup destination")
struct.pack_into(">h", data, code_at + 42, 15 - 41)
output.write_bytes(data)
print("test(I)V BCI 41 goto 63 -> goto 15; external entry to second protected segment")
