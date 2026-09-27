import re
import sys
from pathlib import Path


def method(path):
    text = Path(path).read_text()
    marker = re.search(r"\n  public void test\(\)(?: throws java\.io\.IOException)?;", text)
    if marker is None:
        raise SystemExit(f"missing test() in {path}")
    body = text[marker.end():].split("      LineNumberTable:", 1)[0]
    code, rows, in_table = [], [], False
    for line in body.splitlines():
        insn = re.match(r"\s*(\d+):\s+([a-z][a-z0-9_]*)", line)
        if insn:
            code.append((int(insn.group(1)), insn.group(2)))
        if "Exception table:" in line:
            in_table = True
            continue
        if in_table:
            row = re.match(r"\s*(\d+)\s+(\d+)\s+(\d+)\s+(?:Class )?([^\s]+)", line)
            if row:
                rows.append(tuple(row.groups()))
            elif rows and line.strip():
                in_table = False
    return code, rows


actual = method(sys.argv[1])
fixed = method(sys.argv[2])
print(f"test() BCI/opcode equal: {str(actual[0] == fixed[0]).lower()}")
print(f"test() exception rows equal: {str(actual[1] == fixed[1]).lower()}")
if actual != fixed:
    print(f"recompiled={actual}")
    print(f"pinned={fixed}")
    raise SystemExit(1)
