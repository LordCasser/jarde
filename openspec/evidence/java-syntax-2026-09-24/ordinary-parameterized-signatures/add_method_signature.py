#!/usr/bin/env python3
"""Add a JVMS Signature attribute to one existing method without changing code/descriptors."""
from pathlib import Path
import struct, sys

src, dst, method_name, signature = sys.argv[1:]
data = bytearray(Path(src).read_bytes())
if data[:4] != b'\xca\xfe\xba\xbe': raise SystemExit('not a class file')
cp_count = struct.unpack_from('>H', data, 8)[0]
pos = 10
utf = {}
i = 1
while i < cp_count:
    tag = data[pos]; start = pos; pos += 1
    if tag == 1:
        n = struct.unpack_from('>H', data, pos)[0]; pos += 2
        value = bytes(data[pos:pos+n]).decode('utf-8'); pos += n
        utf[i] = value
    elif tag in (3,4): pos += 4
    elif tag in (5,6): pos += 8; i += 1
    elif tag in (7,8,16,19,20): pos += 2
    elif tag in (9,10,11,12,17,18): pos += 4
    elif tag == 15: pos += 3
    else: raise SystemExit(f'unknown constant pool tag {tag} at {start}')
    i += 1
cp_end = pos
# Locate member table using its exact JVMS layout.
def skip_attributes(offset, count):
    for _ in range(count):
        length = struct.unpack_from('>I', data, offset+2)[0]
        offset += 6 + length
    return offset
pos += 6
interfaces = struct.unpack_from('>H', data, pos)[0]; pos += 2 + interfaces*2
fields = struct.unpack_from('>H', data, pos)[0]; pos += 2
for _ in range(fields):
    attrs = struct.unpack_from('>H', data, pos+6)[0]
    pos = skip_attributes(pos+8, attrs)
methods_count_offset = pos
methods = struct.unpack_from('>H', data, pos)[0]; pos += 2
insert_at = None
attr_count_at = None
for _ in range(methods):
    name_idx, desc_idx, attr_count = struct.unpack_from('>HHH', data, pos+2)
    member_name = utf[name_idx]
    attrs_at = pos+8
    end = skip_attributes(attrs_at, attr_count)
    if member_name == method_name:
        insert_at, attr_count_at = end, pos+6
        break
    pos = end
if insert_at is None: raise SystemExit(f'method {method_name!r} absent')
name_bytes = signature.encode('utf-8')
new_cp = bytearray()
next_index = cp_count
name_index = next_index
new_cp += bytes((1,)) + struct.pack('>H', len(b'Signature')) + b'Signature'
next_index += 1
signature_index = next_index
new_cp += bytes((1,)) + struct.pack('>H', len(name_bytes)) + name_bytes
# Increase pool count; append both UTF8 entries immediately before original class body.
struct.pack_into('>H', data, 8, cp_count+2)
data[cp_end:cp_end] = new_cp
insert_at += len(new_cp); attr_count_at += len(new_cp)
old_count = struct.unpack_from('>H', data, attr_count_at)[0]
struct.pack_into('>H', data, attr_count_at, old_count+1)
attribute = struct.pack('>HIH', name_index, 2, signature_index)
data[insert_at:insert_at] = attribute
Path(dst).write_bytes(data)
