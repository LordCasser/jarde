#!/usr/bin/env python3
"""Retarget one javac bridge caller's receiver to its explicit same-type argument."""
import pathlib
import shutil
import struct
import sys

src = pathlib.Path(sys.argv[1])
dst = pathlib.Path(sys.argv[2])
shutil.copytree(src, dst, dirs_exist_ok=True)
path = dst / 'OuterSuperCases$Member.class'
data = bytearray(path.read_bytes())

def u2(pos): return struct.unpack_from('>H', data, pos)[0]
def u4(pos): return struct.unpack_from('>I', data, pos)[0]

assert u4(0) == 0xCAFEBABE
cp_count = u2(8)
pos = 10
cp = [None] * cp_count
utf8 = {}
for index in range(1, cp_count):
    tag = data[pos]
    pos += 1
    if tag == 1:
        length = u2(pos); pos += 2
        value = bytes(data[pos:pos+length]).decode('utf-8')
        cp[index] = (tag, value)
        utf8[index] = value
        pos += length
    elif tag in (3, 4): pos += 4
    elif tag in (5, 6): pos += 8; index += 1
    elif tag in (7, 8, 16, 19, 20):
        cp[index] = (tag, u2(pos)); pos += 2
    elif tag in (9, 10, 11, 12, 17, 18):
        cp[index] = (tag, u2(pos), u2(pos+2)); pos += 4
    elif tag == 15: cp[index] = (tag, data[pos], u2(pos+1)); pos += 3
    else: raise ValueError(f'unsupported constant-pool tag {tag}')

refs = {}
for index, item in enumerate(cp):
    if item and item[0] == 10:
        cls = cp[item[1]]
        nt = cp[item[2]]
        owner = cp[cls[1]][1]
        name, desc = utf8[nt[1]], utf8[nt[2]]
        refs[(owner, name, desc)] = index
target = refs[('OuterSuperCases', 'access$101', '(LOuterSuperCases;)I')]

pos += 6  # access_flags, this_class, super_class
interfaces = u2(pos); pos += 2 + interfaces * 2

def skip_members(pos):
    count = u2(pos); pos += 2
    for _ in range(count):
        attrs = u2(pos+6); pos += 8
        for _ in range(attrs):
            length = u4(pos+2); pos += 6 + length
    return pos

pos = skip_members(pos)  # fields
method_count = u2(pos); pos += 2
found = False
for _ in range(method_count):
    name = utf8[u2(pos+2)]
    desc = utf8[u2(pos+4)]
    attr_count = u2(pos+6); pos += 8
    for _ in range(attr_count):
        attr_name = utf8[u2(pos)]
        attr_len = u4(pos+2)
        info = pos + 6
        if name == 'otherBridgeCandidate' and desc == '(LOuterSuperCases;)I' and attr_name == 'Code':
            code_len = u4(info+4)
            code_start = info + 8
            code = data[code_start:code_start+code_len]
            assert code[:4] == bytes((0x2a, 0xb4, code[2], code[3]))
            assert code[4] == 0xb8
            # aload_1 replaces aload_0/getfield this$0; three NOPs keep all following BCIs stable.
            data[code_start:code_start+4] = bytes((0x2b, 0x00, 0x00, 0x00))
            struct.pack_into('>H', data, code_start+5, target)
            found = True
        pos += 6 + attr_len
    if found: break
assert found, 'method Code attribute not found'
path.write_bytes(data)
print(f'patched {path.name}: otherBridgeCandidate BCI 0..3 -> aload_1/nop/nop/nop; BCI 4 invokestatic cp#{target} access$101')
