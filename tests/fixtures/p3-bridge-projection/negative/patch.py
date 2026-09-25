from pathlib import Path
import struct
import argparse

parser = argparse.ArgumentParser(description='Patch the second method into a side-effecting bridge')
parser.add_argument('--input', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
path = args.input
b = bytearray(path.read_bytes())
assert b[:4] == b'\xca\xfe\xba\xbe'

def u2(i): return struct.unpack_from('>H', b, i)[0]
def u4(i): return struct.unpack_from('>I', b, i)[0]

i = 8
cp_count = u2(i); i += 2
cp = {}
index = 1
while index < cp_count:
    tag = b[i]; i += 1
    if tag == 1:
        n = u2(i); i += 2
        cp[index] = (i, bytes(b[i:i+n]))
        i += n
    elif tag in (3, 4, 9, 10, 11, 12, 17, 18): i += 4
    elif tag in (5, 6): i += 8
    elif tag in (7, 8, 16, 19, 20): i += 2
    elif tag == 15: i += 3
    else: raise ValueError(tag)
    index += 2 if tag in (5, 6) else 1
assert list(value for _, value in cp.values()).count(b'geh') == 1
name_index = next(index for index, (_, value) in cp.items() if value == b'geh')
name_offset, _ = cp[name_index]
b[name_offset:name_offset+3] = b'get'

i += 6
interfaces = u2(i); i += 2 + interfaces * 2

def skip_members(i):
    count = u2(i); i += 2
    for _ in range(count):
        i += 6
        attributes = u2(i); i += 2
        for _ in range(attributes):
            i += 2
            n = u4(i); i += 4 + n
    return i

i = skip_members(i)  # fields
methods = u2(i); i += 2
patched = 0
for _ in range(methods):
    start = i
    access = u2(i)
    name = u2(i+2)
    descriptor = u2(i+4)
    i += 6
    attributes = u2(i); i += 2
    if name == name_index and cp[descriptor][1] == b'()Ljava/lang/Object;':
        assert access == 1
        struct.pack_into('>H', b, start, access | 0x1040)
        patched += 1
    for _ in range(attributes):
        i += 2
        n = u4(i); i += 4+n
assert patched == 1
patched_path = args.output
patched_path.write_bytes(b)
print(patched_path)
