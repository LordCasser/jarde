#!/usr/bin/env python3
"""Replay one frozen Java 8 anonymous-capture source/bytecode comparison."""

from __future__ import annotations

import hashlib
import json
import shutil
import subprocess
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parent
CLASSES = (
    "AnonymousProbe.class",
    "AnonymousProbe$1.class",
    "AnonymousProbe$Action.class",
)


def run(*args: str, cwd: Path) -> dict[str, object]:
    result = subprocess.run(args, cwd=cwd, capture_output=True, text=True, check=False)
    return {
        "command": list(args),
        "exit": result.returncode,
        "stdout": result.stdout,
        "stderr": result.stderr,
    }


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> None:
    with tempfile.TemporaryDirectory(prefix="jarde-anonymous-capture-") as tmp:
        work = Path(tmp)
        original = work / "original"
        original.mkdir()
        shutil.copy2(ROOT / "AnonymousProbe.java", original / "AnonymousProbe.java")
        original_compile = run(
            "javac", "--release", "8", "-g:none", "-Xlint:-options",
            "AnonymousProbe.java", cwd=original
        )
        assert original_compile["exit"] == 0, original_compile
        rebuilt = {name: digest(original / name) for name in CLASSES}
        frozen = {name: digest(ROOT / name) for name in CLASSES}
        assert rebuilt == frozen, (rebuilt, frozen)
        original_run = run("java", "-Xverify:all", "-cp", str(original), "AnonymousProbe", cwd=work)
        assert original_run["exit"] == 0 and original_run["stdout"] == "10\n", original_run

        jadx = work / "jadx"
        jadx.mkdir()
        shutil.copy2(ROOT / "jadx-AnonymousProbe.java", jadx / "AnonymousProbe.java")
        jadx_compile = run(
            "javac", "--release", "8", "-Xlint:-options", "AnonymousProbe.java", cwd=jadx
        )

        jarde = work / "jarde"
        jarde.mkdir()
        shutil.copy2(ROOT / "jarde-outer.java", jarde / "AnonymousProbe.java")
        jarde_compile = run(
            "javac", "--release", "8", "-Xlint:-options", "-cp", str(original),
            "AnonymousProbe.java", cwd=jarde
        )

        assert jadx_compile["exit"] != 0, jadx_compile
        assert jarde_compile["exit"] != 0, jarde_compile
        print(json.dumps({
            "frozen_sha256": frozen,
            "original_compile": original_compile,
            "original_run": original_run,
            "jadx_compile": jadx_compile,
            "jarde_compile": jarde_compile,
        }, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
