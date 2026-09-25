#!/usr/bin/env python3
"""Show why enum-field names alone do not prove a direct enum switch."""

from __future__ import annotations

import hashlib
import json
import shutil
import subprocess
import sys
from pathlib import Path


HERE = Path(__file__).resolve().parent
CASE = HERE.parents[1]
INPUT = CASE / "patched-map"
HUE_ORIGINAL_SHA = "3a5d7db6f4420d09cbdd0f128dcaaa4759fd33de11196c6bebdb10917db67ab2"
HUE_ALIASED_SHA = "c170d13e3e0bacaed624b7f5c4d4f845135a4b7633ee563040414329a7811d9b"
ORIGINAL = bytes.fromhex("bb000159122204b70021b30007")
ALIASED = bytes.fromhex("b20003") + bytes(7) + bytes.fromhex("b30007")


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def run(*args: str) -> str:
    completed = subprocess.run(args, check=True, capture_output=True, text=True)
    return completed.stdout


def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit("usage: replay.py OUTPUT_DIRECTORY")
    output = Path(sys.argv[1]).resolve()
    output.mkdir(parents=True, exist_ok=True)
    for name in ("Hue.class", "EnumSwitchSubject.class", "EnumSwitchSubject$1.class", "EnumSwitchRunner.class"):
        shutil.copyfile(INPUT / name, output / name)

    hue_path = output / "Hue.class"
    original = hue_path.read_bytes()
    assert sha256(original) == HUE_ORIGINAL_SHA
    assert len(ORIGINAL) == len(ALIASED) and original.count(ORIGINAL) == 1
    aliased = original.replace(ORIGINAL, ALIASED, 1)
    assert sha256(aliased) == HUE_ALIASED_SHA
    hue_path.write_bytes(aliased)

    original_output = run("java", "-Xverify:all", "-cp", str(output), "EnumSwitchRunner")
    projected = output / "projected"
    projected.mkdir(exist_ok=True)
    source = (CASE / "EnumSwitchSubject.java").read_text()
    before = "case RED:\n                return mark(1);\n            case BLUE:\n                return mark(2);"
    after = "case BLUE:\n                return mark(1);\n            case RED:\n                return mark(2);"
    assert source.count(before) == 1
    (projected / "EnumSwitchSubject.java").write_text(source.replace(before, after))
    run("javac", "--release", "8", "-Xlint:-options", "-cp", str(output), "-d", str(projected), str(projected / "EnumSwitchSubject.java"))
    projected_output = run("java", "-Xverify:all", "-cp", f"{projected}:{output}", "EnumSwitchRunner")

    summary = {
        "input_hue_sha256": HUE_ORIGINAL_SHA,
        "aliased_hue_sha256": HUE_ALIASED_SHA,
        "original": original_output.splitlines(),
        "direct_switch": projected_output.splitlines(),
    }
    assert summary["original"] == ["1|1", "1|1", "3|3", "null|0"]
    assert summary["direct_switch"] == ["2|2", "2|2", "3|3", "null|0"]
    (output / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    (output / "aliased-hue-javap.txt").write_text(run("javap", "-c", "-p", str(hue_path)))
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
