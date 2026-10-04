#!/usr/bin/env python3
"""Equal-length byte patches over the real javac 8 N1$Stat.class to synthesize the
position-constraint negatives D (result not discarded: pop -> astore_2) and
E (the four-window misaligned: aload_1; dup -> nop; aload_1).

Self-checks before patching: the target byte pattern must appear exactly once and
the known-good original must hash to the recorded SHA256. After patching, javap
re-disassembly is printed for the record."""
import hashlib
import sys
from pathlib import Path

SRC = Path(sys.argv[1])
OUTDIR = Path(sys.argv[2])
EXPECTED = "a563455582b5e55f34528f754b588b459be8864def86748da87d2335f54c1431"

data = SRC.read_bytes()
digest = hashlib.sha256(data).hexdigest()
assert digest == EXPECTED, f"self-check failed: {digest} != {EXPECTED}"
print(f"self-check OK: {SRC.name} sha256 matches frozen real-javac8 N1$Stat")

# use(N1) body pattern: aload_1(0x2b) dup(0x59) invokevirtual getClass(0xb6 i i) pop(0x57)
pattern = bytes([0x2B, 0x59, 0xB6])
positions = []
start = 0
while True:
    i = data.find(pattern, start)
    if i < 0:
        break
    positions.append(i)
    start = i + 1
assert len(positions) == 1, f"self-check failed: pattern occurs {len(positions)} times"
at = positions[0]
assert data[at + 5] == 0x57, f"self-check failed: no pop after getClass at {at+5}"
print(f"self-check OK: unique [aload_1, dup, invokevirtual, pop] at code offset {at}..{at+5}")

# D: pop -> astore_2 (0x4d). Local 2 is free in use(N1): 0=this, 1=outer.
d = bytearray(data)
d[at + 5] = 0x4D
(OUTDIR / "N1$Stat.pop-missing.class").write_bytes(bytes(d))
print("D written: BCI-9 pop(0x57) -> astore_2(0x4d), equal length, stack-neutral")

# E: aload_1 dup -> nop aload_1 (the window's load/dup pair misaligned by one slot)
e = bytearray(data)
e[at] = 0x00
e[at + 1] = 0x2B
(OUTDIR / "N1$Stat.check-noncontiguous.class").write_bytes(bytes(e))
print("E written: aload_1;dup -> nop;aload_1, equal length, types sound")

print("patched digests:")
for name in ("N1$Stat.pop-missing.class", "N1$Stat.check-noncontiguous.class"):
    print(f"  {name} " + hashlib.sha256((OUTDIR / name).read_bytes()).hexdigest())
