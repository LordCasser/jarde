#!/usr/bin/env python3
"""Corpus two-leg scan: render every frozen proved-java-structure fixture class with one jarde
binary. Usage: scan.py <jarde-binary> <output-dir>. One jar per fixture directory, one
`class-source --policy plain-jar --release 8 --format text` request per class. The exit status of
each request is recorded; renderings are written per class for a later byte diff."""

import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

REPO = Path(__file__).resolve().parents[5]
FIXTURE_ROOT = REPO / "tests" / "fixtures" / "proved-java-structure"


def main() -> None:
    if len(sys.argv) != 3:
        raise SystemExit("usage: scan.py <jarde-binary> <output-dir>")
    cli, out_dir = Path(sys.argv[1]), Path(sys.argv[2])
    out_dir.mkdir(parents=True, exist_ok=True)
    summary = []
    for fixture in sorted(p for p in FIXTURE_ROOT.iterdir() if p.is_dir()):
        classes = sorted(fixture.rglob("*.class"))
        if not classes:
            continue
        with tempfile.TemporaryDirectory() as work:
            work_dir = Path(work)
            class_dir = work_dir / "classes"
            class_dir.mkdir()
            for path in classes:
                shutil.copyfile(path, class_dir / path.name)
            jar = work_dir / "fixture.jar"
            packed = subprocess.run(
                ["jar", "--create", "--file", str(jar), "-C", str(class_dir), "."],
                capture_output=True, text=True,
            )
            if packed.returncode:
                raise SystemExit(f"jar failed for {fixture.name}: {packed.stderr}")
            listing = subprocess.run(
                [str(cli), "list-classes", "--input", str(jar), "--format", "json"],
                capture_output=True, text=True,
            )
            import json

            document = json.loads(listing.stdout)
            names = sorted(
                item["declaration"]["this_class"]["escaped"] for item in document["items"]
            )
            for name in names:
                label = (
                    f"{fixture.name}::"
                    + name.replace("$", "_DOLLAR_").replace("/", "_SLASH_")
                )
                request = subprocess.run(
                    [str(cli), "class-source", "--input", str(jar), "--class", name,
                     "--policy", "plain-jar", "--release", "8", "--format", "text"],
                    capture_output=True, text=True,
                )
                (out_dir / f"{label}.java").write_text(request.stdout)
                summary.append(f"{label}\texit={request.returncode}")
    (out_dir / "SUMMARY.txt").write_text("".join(f"{line}\n" for line in summary))
    print(f"{len(summary)} renderings written to {out_dir}")


if __name__ == "__main__":
    main()
