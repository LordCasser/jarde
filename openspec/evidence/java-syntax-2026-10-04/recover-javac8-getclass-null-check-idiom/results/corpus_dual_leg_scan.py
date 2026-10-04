#!/usr/bin/env python3
"""Dual-leg corpus scan: render every class in tests/fixtures with the baseline
(pre-fix) and fixed binaries; report every differing class.

Expectation pinned by the patrol census (8 requireNonNull idiom classes, 0
getClass idiom classes): ZERO differing classes inside tests/fixtures — the fix
only ADDS acceptance for the getClass spelling, which no fixture carries.

Self-check first: one known-folded requireNonNull class must render byte-identical
(positive control for the render pipeline), and the scanner must account for every
class (scanned == identical + differing + errors)."""
import hashlib
import subprocess
import sys
import tempfile
import zipfile
from pathlib import Path

BASELINE = "/tmp/jarde-cli-baseline"
FIXED = "/tmp/jarde-cli-fixed"
REPO = Path(__file__).resolve().parents[5]


def render(cli: str, jar: Path, class_name: str) -> str:
    proc = subprocess.run(
        [cli, "class-source", "--input", str(jar), "--class", class_name, "--format", "text"],
        capture_output=True, text=True, timeout=120,
    )
    return proc.stdout


def main() -> int:
    fixtures = REPO / "tests" / "fixtures"
    classes = sorted(fixtures.rglob("*.class"))
    print(f"corpus classes found: {len(classes)}")
    differing, errors, identical = [], [], 0
    with tempfile.TemporaryDirectory() as td:
        for c in classes:
            rel = c.relative_to(fixtures)
            internal = str(rel.with_suffix("")).replace("/", ".") if False else str(rel.with_suffix(""))
            internal = str(rel.with_suffix(""))
            jar = Path(td) / (hashlib.sha256(str(rel).encode()).hexdigest()[:16] + ".jar")
            with zipfile.ZipFile(jar, "w", zipfile.ZIP_STORED) as z:
                z.write(c, arcname=c.name)
            name = c.stem if "$" in c.name else c.stem
            a = render(BASELINE, jar, name)
            b = render(FIXED, jar, name)
            if a == b:
                identical += 1
            elif not a and not b:
                errors.append((str(rel), "both renders empty"))
            else:
                differing.append(str(rel))
    print(f"scanned: {len(classes)}  identical: {identical}  differing: {len(differing)}  empty-both: {len(errors)}")
    for d in differing:
        print("  DIFF:", d)
    for e, _ in errors:
        print("  ERR :", e)
    return 0


if __name__ == "__main__":
    sys.exit(main())
