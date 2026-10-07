#!/usr/bin/env python3
"""Classify the stderr deltas of one two-leg capture tree (task 3.1).

Every capture writes the report's bookkeeping planes to standard error, so two legs differ on
`elapsed_millis` by construction. This script drops that field and reports what is left, per
capture:

* `timing-only` — nothing beyond `elapsed_millis`;
* `ir_items-only` — every remaining difference is an `ir_items` line: the recovery layer's own
  charge for retaining the body's AST (the double-brace allocation point's cheap pre-filter);
* `probe-work` — the differences include the read/analysis usage of a probe (a class the
  admission read and did not claim): classified by hand from the report, the presentation itself
  unchanged;
* `other` — anything else, printed for the reviewer.

    python3 classify-stderr.py <a-dir> <b-dir>
"""

import difflib
import pathlib
import re
import sys


def stripped(text: str) -> list[str]:
    return [re.sub(r"elapsed_millis = \d+", "elapsed_millis = X", line) for line in text.splitlines()]


def main() -> int:
    a, b = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
    counts: dict[str, int] = {}
    others: list[str] = []
    for path in sorted(a.glob("*.err")):
        other = b / path.name
        left, right = stripped(path.read_text()), stripped(other.read_text())
        if left == right:
            continue
        lines = [
            line
            for line in difflib.unified_diff(left, right, lineterm="", n=0)
            if not line.startswith(("---", "+++", "@@"))
        ]
        if all("ir_items = " in line for line in lines):
            kind = "ir_items-only"
        else:
            kind = "other"
            others.append(f"{path.name}: " + " | ".join(line.strip() for line in lines[:4]))
        counts[kind] = counts.get(kind, 0) + 1
    print("stderr captures:", len(list(a.glob("*.err"))), "differing:", sum(counts.values()))
    for kind, count in sorted(counts.items()):
        print(f"  {kind}: {count}")
    for line in others[:40]:
        print("  OTHER", line)
    return 0


if __name__ == "__main__":
    sys.exit(main())
