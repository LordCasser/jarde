#!/usr/bin/env python3
"""Patch only the array descriptor, newarray atype and iastore opcodes."""
import argparse
import hashlib
import json
import struct
from pathlib import Path


def u2(data, pos): return struct.unpack_from(">H", data, pos)[0]
def u4(data, pos): return struct.unpack_from(">I", data, pos)[0]
def sha(data): return hashlib.sha256(data).hexdigest()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("source", type=Path)
    parser.add_argument("destination", type=Path)
    parser.add_argument("manifest", type=Path)
    parser.add_argument("--stores", type=int, default=5)
    parser.add_argument("--descriptor", default="()[I")
    args = parser.parse_args()
    original = args.source.read_bytes()
    data = bytearray(original)
    if original[:4] != b"\xca\xfe\xba\xbe" or u2(data, 6) != 52:
        raise SystemExit("expected Java 8 classfile")
    cp_count = u2(data, 8)
    cp = [None] * cp_count
    spans = [None] * cp_count
    pos, index = 10, 1
    while index < cp_count:
        tag = data[pos]
        pos += 1
        if tag == 1:
            size = u2(data, pos)
            pos += 2
            cp[index] = bytes(data[pos:pos + size])
            spans[index] = (pos, size)
            pos += size
        elif tag in (3, 4): pos += 4
        elif tag in (5, 6): pos += 8; index += 1
        elif tag in (7, 8, 16, 19, 20): pos += 2
        elif tag in (9, 10, 11, 12, 17, 18): pos += 4
        elif tag == 15: pos += 3
        else: raise SystemExit(f"unknown constant pool tag {tag}")
        index += 1
    cp_end = pos
    descriptor_edits = []
    old = args.descriptor.encode()
    if not old.endswith(b"[I"):
        raise SystemExit("input descriptor must return int[]")
    new = old[:-1] + b"Z"
    for old, new in ((old, new),):
        hits = [(i, span) for i, (value, span) in enumerate(zip(cp, spans)) if value == old]
        if not hits or len(old) != len(new): raise SystemExit(f"expected descriptor {old!r}")
        for _, (start, size) in hits:
            data[start:start + size] = new
        descriptor_edits.append({"from": old.decode(), "to": new.decode(), "entries": len(hits)})

    pos = cp_end + 6
    interfaces = u2(data, pos); pos += 2 + 2 * interfaces
    fields = u2(data, pos); pos += 2
    def skip_member(pos):
        count = u2(data, pos + 6); pos += 8
        for _ in range(count): pos += 6 + u4(data, pos + 2)
        return pos
    for _ in range(fields): pos = skip_member(pos)
    methods = u2(data, pos); pos += 2
    target = None
    for _ in range(methods):
        name_index, desc_index, attr_count = u2(data, pos + 2), u2(data, pos + 4), u2(data, pos + 6)
        method_start = pos
        pos += 8
        for _ in range(attr_count):
            name = cp[u2(data, pos)]
            length = u4(data, pos + 2)
            body = pos + 6
            if cp[name_index] == b"values" and cp[desc_index] == args.descriptor.encode() and name == b"Code":
                target = (body, u4(data, body + 4))
            pos = body + length
        if method_start >= pos: raise SystemExit("bad method table")
    if target is None: raise SystemExit("values() Code attribute not found")
    code_start = target[0] + 8
    code_length = target[1]
    code = bytes(data[code_start:code_start + code_length])
    # Javac emits one newarray atype byte and one iastore per initializer element.
    newarrays = [i for i, op in enumerate(code) if op == 0xBC]
    stores = [i for i, op in enumerate(code) if op == 0x4F]
    if len(newarrays) != 1 or len(stores) != args.stores:
        raise SystemExit(f"unexpected Code layout: {code.hex()}, newarray={newarrays}, iastore={stores}")
    if code[newarrays[0] + 1] != 10:
        raise SystemExit("newarray did not allocate int[]")
    data[code_start + newarrays[0] + 1] = 4
    for i in stores: data[code_start + i] = 0x54
    args.destination.parent.mkdir(parents=True, exist_ok=True)
    args.destination.write_bytes(data)
    manifest = {
        "major_version": 52, "input": str(args.source), "input_sha256": sha(original),
        "output": str(args.destination), "output_sha256": sha(data),
        "bytes_equal": len(data) == len(original), "descriptor_edits": descriptor_edits,
        "method": "values" + new.decode(), "original_code_hex": code.hex(),
        "patched_code_hex": bytes(data[code_start:code_start + code_length]).hex(),
        "newarray_atype_patch": {"code_offset": newarrays[0] + 1, "from": 10, "to": 4},
        "store_opcode_patches": [{"code_offset": i, "from": "0x4f", "to": "0x54"} for i in stores],
    }
    args.manifest.parent.mkdir(parents=True, exist_ok=True)
    args.manifest.write_text(json.dumps(manifest, indent=2) + "\n")
    print(json.dumps(manifest, indent=2))

if __name__ == "__main__": main()
