#!/usr/bin/env python3
"""Apply a two-site, byte-length-preserving Code patch to javac output; no assembler dependency."""
import argparse
import struct
from pathlib import Path


def u2(data, at): return struct.unpack_from('>H', data, at)[0]
def u4(data, at): return struct.unpack_from('>I', data, at)[0]


def cp_and_methods(data):
    if data[:4] != b'\xca\xfe\xba\xbe': raise ValueError('not a class file')
    count = u2(data, 8); cp = [None] * count; pos = 10; i = 1
    while i < count:
        tag = data[pos]; pos += 1
        if tag == 1:
            n = u2(data, pos); pos += 2
            cp[i] = data[pos:pos+n].decode('ascii'); pos += n
        elif tag in (3, 4): pos += 4
        elif tag in (5, 6): pos += 8; i += 1
        elif tag == 7: cp[i] = ('class', u2(data, pos)); pos += 2
        elif tag == 12: cp[i] = ('nat', u2(data, pos), u2(data, pos+2)); pos += 4
        elif tag in (10, 11): cp[i] = ('methodref', u2(data, pos), u2(data, pos+2)); pos += 4
        elif tag in (9, 17, 18): pos += 4
        elif tag in (8, 16, 19, 20): pos += 2
        elif tag == 15: pos += 3
        else: raise ValueError(f'unsupported constant-pool tag {tag}')
        i += 1
    pos += 6
    interfaces = u2(data, pos); pos += 2 + 2 * interfaces
    def skip_members(pos, is_method):
        n = u2(data, pos); pos += 2; found = {}
        for _ in range(n):
            _, name_i, desc_i, attrs = struct.unpack_from('>HHHH', data, pos); pos += 8
            name, desc = cp[name_i], cp[desc_i]
            for _ in range(attrs):
                attr_name_i = u2(data, pos); length = u4(data, pos + 2)
                attr_name, body = cp[attr_name_i], pos + 6
                if is_method and attr_name == 'Code':
                    code_len = u4(data, body + 4); code_at = body + 8
                    key = (name, desc)
                    if key in found: raise ValueError(f'duplicate Code {key}')
                    found[key] = (code_at, code_len)
                pos = body + length
        return pos, found
    pos, _ = skip_members(pos, False)
    pos, methods = skip_members(pos, True)
    return cp, methods


def methodref(cp, owner, name, descriptor):
    matches = []
    for index, entry in enumerate(cp):
        if not isinstance(entry, tuple) or entry[0] != 'methodref': continue
        _, class_i, nat_i = entry
        class_entry, nat_entry = cp[class_i], cp[nat_i]
        if class_entry[0] != 'class' or nat_entry[0] != 'nat': continue
        owner_name = cp[class_entry[1]]
        method_name, method_desc = cp[nat_entry[1]], cp[nat_entry[2]]
        if (owner_name, method_name, method_desc) == (owner, name, descriptor):
            matches.append(index)
    if len(matches) != 1: raise ValueError(f'expected one {owner}.{name}{descriptor} reference, found {matches}')
    return matches[0]


def patch_method(data, cp, methods, key, cp_g):
    if key not in methods: raise ValueError(f'missing method Code {key}')
    at, length = methods[key]; code = bytes(data[at:at+length])
    invoke_g = b'\xb8' + struct.pack('>H', cp_g)
    matches = []
    for store in range(0x3b, 0x3f):
        needle = b'\x04\x64' + bytes([store]) + invoke_g  # iconst_1; isub; istore_n; invokestatic g
        start = 0
        while True:
            found = code.find(needle, start)
            if found < 0: break
            matches.append((found, needle, b'\x04\x64' + invoke_g + bytes([store])))
            start = found + 1
    if len(matches) != 1: raise ValueError(f'{key}: expected one computed-init/store/g sequence, found {len(matches)}')
    found, before, after = matches[0]
    changed = code[:found] + after + code[found+len(before):]
    if len(changed) != length: raise AssertionError('patch unexpectedly changed Code length')
    data[at:at+length] = changed
    return {'method': key[0], 'descriptor': key[1], 'bci': found, 'before': before.hex(), 'after': after.hex()}


def main():
    p = argparse.ArgumentParser()
    p.add_argument('input', type=Path, help='javac-produced ComputedForControls.class')
    p.add_argument('output', type=Path, help='separate patched class path')
    args = p.parse_args()
    data = bytearray(args.input.read_bytes())
    cp, methods = cp_and_methods(data)
    g_ref = methodref(cp, 'ComputedForControls', 'g', '()V')
    patches=[]
    for method in [('find','([III)I'),('shared','(I)I')]:
        patches.append(patch_method(data,cp,methods,method,g_ref))
    args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_bytes(data)
    print({'patched_methods':patches,'output_bytes':len(data)})

if __name__=='__main__': main()
