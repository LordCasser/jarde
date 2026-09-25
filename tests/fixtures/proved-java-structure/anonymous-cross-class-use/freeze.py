#!/usr/bin/env python3
"""Compile the source baseline and freeze the single-UTF8 cross-class mutation."""
from pathlib import Path
import hashlib
import shutil
import struct
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parent

def u2(data, at): return struct.unpack_from(">H", data, at)[0]

def replace_utf8(data):
    assert data[:4] == b"\xca\xfe\xba\xbe"
    count = u2(data, 8)
    at, index, hits = 10, 1, 0
    out = bytearray(data)
    while index < count:
        tag = data[at]
        at += 1
        if tag == 1:
            length = u2(data, at)
            start = at + 2
            if data[start:start + length] == b"Other$1":
                out[start:start + length] = b"Owner$1"
                hits += 1
            at = start + length
        elif tag in (3, 4): at += 4
        elif tag in (5, 6): at += 8; index += 1
        elif tag in (7, 8, 16, 19, 20): at += 2
        elif tag in (9, 10, 11, 12, 17, 18): at += 4
        elif tag == 15: at += 3
        else: raise ValueError(f"unexpected constant-pool tag {tag}")
        index += 1
    assert hits == 1, f"expected one Other$1 UTF8 constant, found {hits}"
    assert bytes(out).count(b"Other$1") == data.count(b"Other$1") - 1
    return bytes(out)

def run(args, **kwargs):
    return subprocess.run(args, check=True, text=True, capture_output=True, **kwargs)

with tempfile.TemporaryDirectory(prefix="jarde-anonymous-cross-class-") as temp:
    classes = Path(temp) / "classes"
    classes.mkdir()
    run(["javac", "--release", "8", "-g:none", "-d", str(classes), *map(str, sorted(ROOT.glob("*.java")))])
    baseline = run(["java", "-Xverify:all", "-cp", str(classes), "Main"])
    assert baseline.stdout == "sameClass=false\n", baseline.stdout
    other = classes / "Other.class"
    other.write_bytes(replace_utf8(other.read_bytes()))
    mutated = run(["java", "-Xverify:all", "-cp", str(classes), "Main"])
    assert mutated.stdout == "sameClass=true\n", mutated.stdout
    for source in sorted(classes.glob("*.class")):
        target = ROOT / source.name
        shutil.copyfile(source, target)
        print(f"{hashlib.sha256(target.read_bytes()).hexdigest()}  {target.name}")
    print(f"source: {baseline.stdout.strip()}")
    print(f"mutated: {mutated.stdout.strip()}")
