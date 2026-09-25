import sys
from pathlib import Path

if len(sys.argv) != 3:
    raise SystemExit(f"usage: {sys.argv[0]} SOURCE.class TARGET.class")
source = Path(sys.argv[1])
target = Path(sys.argv[2])
data = bytearray(source.read_bytes())
pattern = bytes((0x2A, 0xC0))
offsets = [offset for offset in range(len(data)) if data.startswith(pattern, offset)]
if len(offsets) != 1:
    raise SystemExit(f"expected one aload_0/checkcast pattern, found {len(offsets)}")
offset = offsets[0]
if data[offset + 4] != 0xB8:
    raise SystemExit(f"expected invokestatic after checkcast at {offset}, found 0x{data[offset + 4]:02x}")
data[offset + 1 : offset + 4] = b"\x00\x00\x00"
target.write_bytes(data)
print(f"patched checkcast at code-file offset {offset}: c0 index -> nop nop nop")
