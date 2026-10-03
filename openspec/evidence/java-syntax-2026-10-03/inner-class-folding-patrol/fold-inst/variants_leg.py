#!/usr/bin/env python3
"""Capture one leg of the instance-fold variant snapshots: every class of every fixture
family, as `class-source --format text`. Usage: variants_leg.py <jarde-cli> <outdir>"""
import subprocess
import sys
from pathlib import Path

cli = Path(sys.argv[1]).resolve()
outdir = Path(sys.argv[2])
outdir.mkdir(parents=True, exist_ok=True)
root = Path(__file__).resolve().parent
families = [
    ("fixture-variants-inst/IV1/IV1.jar", ["IV1", "IV1$Inner"]),
    ("fixture-variants-inst/IV2/IV2.jar", ["IV2", "IV2$Inner"]),
    ("fixture-variants-inst/IV3/IV3.jar", ["IV3", "IV3$B", "IV3$B$C"]),
    ("fixture-variants-inst/IV4/IV4.jar", ["IV4", "IV4$Inner"]),
    ("../fixture/fam.jar", ["N1", "N1$Inner", "N1$Stat"]),
    ("../fold-mix/fixture-variants/MV1.jar", ["MV1", "MV1$Inner"]),
    ("../fold-mix/fixture-variants/MV2.jar", ["MV2", "MV2$Inner"]),
    ("../fold-mix/fixture-variants/MV3.jar", ["MV3", "MV3$Inner"]),
]
for jar_rel, classes in families:
    jar = (root / jar_rel).resolve()
    for internal in classes:
        answer = subprocess.run(
            [str(cli), "class-source", "--input", str(jar), "--class", internal, "--format", "text"],
            capture_output=True,
        )
        if answer.returncode != 0:
            sys.exit(f"{jar}:{internal} exited {answer.returncode}")
        name = internal.replace("$", "_")
        (outdir / f"{name}-inst.txt").write_bytes(answer.stdout)
print(f"captured {sum(len(c) for _, c in families)} units into {outdir}")
