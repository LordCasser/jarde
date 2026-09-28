#!/usr/bin/env python3
import re
import sys


def shape(path):
	text = open(path, encoding="utf-8").read()
	start = text.index("  public void test(java.io.OutputStream)")
	end = text.index("      StackMapTable:", start)
	body = text[start:end]
	instructions = []
	for line in body.splitlines():
		match = re.match(r"\s*(\d+):\s+([a-z][a-z0-9_]*)", line)
		if match:
			instructions.append((int(match.group(1)), match.group(2)))
	rows = [tuple(map(int, row)) for row in re.findall(r"^\s*(\d+)\s+(\d+)\s+(\d+)\s+any$", body, re.M)]
	return instructions, rows


paths = sys.argv[1:]
shapes = [shape(path) for path in paths]
expected_bcIs = [bci for bci, _ in shapes[0][0]]
if not all([bci for bci, _ in insns] == expected_bcIs for insns, _ in shapes[1:]):
	raise SystemExit("instruction BCI sequence differs")
if not all(rows == shapes[0][1] for _, rows in shapes[1:]):
	raise SystemExit("exception table differs")
opcodes = [{bci: opcode for bci, opcode in insns} for insns, _ in shapes]
diffs = [(bci, [row.get(bci) for row in opcodes]) for bci in expected_bcIs
		 if len({row.get(bci) for row in opcodes}) != 1]
if diffs != [(55, ["invokevirtual", "invokespecial", "invokespecial"])]:
	raise SystemExit(f"unexpected opcode differences: {diffs}")
if shapes[0][1] != [(9, 153, 160), (160, 162, 160)]:
	raise SystemExit(f"unexpected catch-all rows: {shapes[0][1]}")
print("matching instruction BCI sequence and catch-all rows across fixed class, standalone Java 8 source, and Java 8 JADX source")
print("single compiler opcode difference: BCI 55 private writeString call: fixed javac invokevirtual; Java 8 javac invokespecial")
