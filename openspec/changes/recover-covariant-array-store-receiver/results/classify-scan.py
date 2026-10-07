#!/usr/bin/env python3
"""Classify one two-leg scan's `diff -r` report (change `recover-covariant-array-store-receiver`).

The scan captures each render twice — once through the **baseline** binary (HEAD) and once through
the **change** binary — and `diff -r` reports every pair that differs. Three kinds of difference
exist, and only one of them is a render delta:

* **render**: a `.txt` pair — the presentation itself. Every one of these is classified by name;
* **plane**: a `.err` pair that differs beyond `elapsed_millis` (segment counts, `text_bytes`,
  `text_digest`, `output_bytes`, …). A render's own bookkeeping moves with the render, so every
  plane delta must belong to a class that also has a render delta; one that does not is a finding;
* **timing**: a `.err` pair whose only differences are `elapsed_millis` lines (no two runs share a
  duration).

Usage: classify-scan.py <diff.txt> [<expected-render-class-substring> ...]
"""

import pathlib
import re
import sys

HEADER = re.compile(r"^diff -r (\S+) (\S+)$")


def pairs(path):
    """The differing pairs and their content lines, read straight out of `diff -r`'s report."""
    current = None
    for line in pathlib.Path(path).read_text(encoding="utf-8", errors="replace").splitlines():
        match = HEADER.match(line)
        if match:
            if current is not None:
                yield current
            current = (match.group(1), match.group(2), [])
        elif current is not None and line[:1] in "<>":
            current[2].append(line)
    if current is not None:
        yield current


def main():
    report = pathlib.Path(sys.argv[1])
    expected = sys.argv[2:]
    render, plane, timing = [], [], 0
    for left, right, lines in pairs(report):
        content = [line for line in lines if "elapsed_millis" not in line]
        if left.endswith(".txt"):
            render.append(left)
        elif content:
            plane.append((left, content))
        else:
            timing += 1
    print(f"{report.name}: render {len(render)}, plane {len(plane)}, timing {timing}")
    print(f"render deltas ({len(render)}):")
    for left in render:
        named = any(token in left for token in expected)
        print(f"  {'classified  ' if named else 'UNCLASSIFIED'} {left}")
    print(f"plane deltas ({len(plane)}):")
    for left, _ in plane:
        paired = left[: -len(".err")] + ".txt" in render
        print(f"  {'with a render delta   ' if paired else 'WITHOUT a render delta'} {left}")


if __name__ == "__main__":
    main()
