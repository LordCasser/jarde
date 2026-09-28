#!/usr/bin/env python3
import re
import sys


def method(path):
    text = open(path, encoding="utf-8").read()
    match = re.search(
        r"public void test\(java\.io\.FileInputStream\);(.*?)LineNumberTable:",
        text,
        re.S,
    )
    if not match:
        raise SystemExit(f"missing test(FileInputStream) code in {path}")
    body = match.group(1)
    instructions = []
    for line in body.splitlines():
        instruction = re.match(r"\s*(\d+):\s+(\S+)(?:\s+(.*?))?\s*$", line)
        if instruction:
            operand = (instruction.group(3) or "").split("//", 1)[0].strip()
            instructions.append((int(instruction.group(1)), instruction.group(2), operand))
    table = re.search(r"Exception table:\s*from\s+to\s+target\s+type(.*)", body, re.S)
    if not table:
        raise SystemExit(f"missing exception table in {path}")
    rows = []
    for line in table.group(1).splitlines():
        row = re.match(r"\s*(\d+)\s+(\d+)\s+(\d+)\s+(?:Class\s+(\S+)|(any))\s*$", line)
        if row:
            rows.append((int(row.group(1)), int(row.group(2)), int(row.group(3)), row.group(4) or "any"))
    return instructions, rows


fixed = method(sys.argv[1])
standalone = method(sys.argv[2])
pinned = method(sys.argv[3])
if fixed != standalone:
    raise SystemExit("fixed and standalone Java 8 method code or exception rows differ")
expected_pinned = (
    [
        (0, "aload_1", ""),
        (1, "invokevirtual", "#7"),
        (4, "goto", "8"),
        (7, "astore_2", ""),
        (8, "return", ""),
    ],
    [(0, 4, 7, "java/io/IOException")],
)
if pinned != expected_pinned:
    raise SystemExit(f"pinned JADX source bytecode differs from its expected source shape: {pinned!r}")

print("fixed_equals_standalone_method_code_and_exception_rows=true")
print("fixed_and_standalone_instructions=")
for bci, opcode, operand in fixed[0]:
    print(f"  {bci}: {opcode}" + (f" {operand}" if operand else ""))
print("fixed_and_standalone_exception_rows=")
for start, end, target, catch_type in fixed[1]:
    print(f"  [{start},{end}) -> {target} {catch_type}")
print("pinned_jadx_instructions=")
for bci, opcode, operand in pinned[0]:
    print(f"  {bci}: {opcode}" + (f" {operand}" if operand else ""))
print("pinned_jadx_exception_rows=")
for start, end, target, catch_type in pinned[1]:
    print(f"  [{start},{end}) -> {target} {catch_type}")
