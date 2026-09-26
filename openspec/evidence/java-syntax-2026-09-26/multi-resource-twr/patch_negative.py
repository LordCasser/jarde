"""Patch only the outer TWR row's end_pc, retaining instruction alignment and frames."""
from pathlib import Path
import shutil
import struct

base = Path(__file__).resolve().parent
source = base / "release8"
out = base / "patched-negative"
out.mkdir(exist_ok=True)
path = out / "MultiResourceTwr.class"
shutil.copy2(source / path.name, path)
data = bytearray(path.read_bytes())

def u1(pos): return data[pos]
def u2(pos): return struct.unpack_from(">H", data, pos)[0]
def u4(pos): return struct.unpack_from(">I", data, pos)[0]

pos = 8
cp_count = u2(pos); pos += 2
utf8 = {}
i = 1
while i < cp_count:
    tag = u1(pos); pos += 1
    if tag == 1:
        length = u2(pos); pos += 2
        utf8[i] = bytes(data[pos:pos + length]).decode("utf-8", "replace")
        pos += length
    elif tag in (3, 4): pos += 4
    elif tag in (5, 6): pos += 8; i += 1
    elif tag in (7, 8, 16, 19, 20): pos += 2
    elif tag in (9, 10, 11, 12, 17, 18): pos += 4
    elif tag == 15: pos += 3
    else: raise ValueError(f"unknown constant pool tag {tag}")
    i += 1
pos += 6
interfaces = u2(pos); pos += 2 + 2 * interfaces
fields = u2(pos); pos += 2
for _ in range(fields):
    attr_count = u2(pos + 6); pos += 8
    for _ in range(attr_count):
        size = u4(pos + 2); pos += 6 + size
methods = u2(pos); pos += 2
patched = False
for _ in range(methods):
    name_index = u2(pos + 2)
    attr_count = u2(pos + 6)
    pos += 8
    for _ in range(attr_count):
        attr_name = utf8[u2(pos)]
        length = u4(pos + 2)
        body = pos + 6
        if name_index in utf8 and utf8[name_index] == "run" and attr_name == "Code":
            code_length = u4(body + 4)
            table = body + 8 + code_length
            count = u2(table); table += 2
            if count != 5: raise ValueError(f"expected 5 TWR rows, got {count}")
            # Entry 2 is outer row [10,37)->59. Its close-start endpoint must be 37.
            entry = table + 2 * 8
            start, end, target, catch = struct.unpack_from(">HHHH", data, entry)
            if (start, end, target) != (10, 37, 59):
                raise ValueError(f"unexpected outer row {(start, end, target)}")
            struct.pack_into(">H", data, entry + 2, 33)
            patched = True
        pos += 6 + length
    if patched: break
if not patched: raise ValueError("run Code attribute not found")
path.write_bytes(data)
