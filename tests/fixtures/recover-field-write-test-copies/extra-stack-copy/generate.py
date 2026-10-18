"""Rewrite one exact opcode in a pinned complete class; class facts are reviewed separately."""
import argparse, hashlib
from pathlib import Path
ap=argparse.ArgumentParser();ap.add_argument('input_class',type=Path);ap.add_argument('output_class',type=Path);args=ap.parse_args()
data=args.input_class.read_bytes()
assert hashlib.sha256(data).hexdigest()=='378fdbe1e8bcc06b163feb34d364e7a49358f70fb645e3b37a80620e89555a1a'
code=bytes.fromhex('2a2a1bb7000c5ab5000d08a2000704a7000403ac')
assert data.count(code)==1
at=data.index(code)+10
assert data[at]==0x08
changed=data[:at]+b'\x59'+data[at+1:]
assert len(changed)==len(data) and sum(a!=b for a,b in zip(data,changed))==1
args.output_class.parent.mkdir(parents=True,exist_ok=True);args.output_class.write_bytes(changed)
print('FieldWriteCopies.assignAndCompare(I)Z BCI10 iconst_5 -> dup, one byte, complete class retained')
