#!/usr/bin/env python3
"""Freeze two verifier-valid enum array counterexamples from the Java 8 baseline."""

from __future__ import annotations

import hashlib
import json
import shutil
import subprocess
import sys
import zipfile
from pathlib import Path


HERE = Path(__file__).resolve().parent
CASE = HERE.parents[1]
INPUT = CASE / "original"
HUE_SHA256 = "3a5d7db6f4420d09cbdd0f128dcaaa4759fd33de11196c6bebdb10917db67ab2"


def run(*args: str) -> str:
    return subprocess.run(args, check=True, capture_output=True, text=True).stdout


def replace_once(data: bytes, old: bytes, new: bytes) -> bytes:
    assert len(old) == len(new) and data.count(old) == 1
    return data.replace(old, new, 1)


def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit("usage: replay.py OUTPUT_DIRECTORY")
    output = Path(sys.argv[1]).resolve()
    output.mkdir(parents=True, exist_ok=True)
    baseline = (INPUT / "Hue.class").read_bytes()
    assert hashlib.sha256(baseline).hexdigest() == HUE_SHA256
    projected = output / "direct-enum-source"
    projected.mkdir(exist_ok=True)
    run("javac", "--release", "8", "-Xlint:-options", "-d", str(projected),
        str(CASE / "Hue.java"), str(CASE / "EnumSwitchSubject.java"), str(HERE / "ValuesRunner.java"))
    direct_source = run("java", "-Xverify:all", "-cp", str(projected), "ValuesRunner").splitlines()
    assert direct_source == ["RED,BLUE,GREEN", "1,2,3"]
    summary = {"direct_enum_source": direct_source}
    for name, hue in {
        "factory-null-element": replace_once(
            baseline,
            bytes.fromhex("5905b2000a53b0"),
            bytes.fromhex("590501000053b0"),
        ),
        "values-returns-null": replace_once(
            baseline,
            bytes.fromhex("b2000db60011c00012b0"),
            bytes.fromhex("010000000000000000b0"),
        ),
    }.items():
        directory = output / name
        directory.mkdir(exist_ok=True)
        for class_name in ("EnumSwitchSubject.class", "EnumSwitchSubject$1.class"):
            shutil.copyfile(INPUT / class_name, directory / class_name)
        (directory / "Hue.class").write_bytes(hue)
        with zipfile.ZipFile(output / f"{name}.jar", "w", compression=zipfile.ZIP_STORED) as archive:
            for class_name in ("Hue.class", "EnumSwitchSubject.class", "EnumSwitchSubject$1.class"):
                entry = zipfile.ZipInfo(class_name, date_time=(1980, 1, 1, 0, 0, 0))
                entry.compress_type = zipfile.ZIP_STORED
                archive.writestr(entry, (directory / class_name).read_bytes())
        run("javac", "--release", "8", "-Xlint:-options", "-cp", str(directory), "-d", str(directory), str(HERE / "ValuesRunner.java"))
        observed = run("java", "-Xverify:all", "-cp", str(directory), "ValuesRunner").splitlines()
        summary[name] = {"hue_sha256": hashlib.sha256(hue).hexdigest(), "observed": observed}
        (directory / "Hue.javap.txt").write_text(run("javap", "-c", "-p", str(directory / "Hue.class")))
    assert summary["factory-null-element"]["observed"] == ["RED,BLUE,null", "1,2,3"]
    assert summary["values-returns-null"]["observed"] == ["null", "ExceptionInInitializerError"]
    (output / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
