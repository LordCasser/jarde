#!/usr/bin/env python3
"""Compile and compare complete Java 8 annotation classes under a frozen Jarde CLI."""
from __future__ import annotations

import hashlib
import json
import os
import shutil
import subprocess
from pathlib import Path

HERE = Path(__file__).resolve().parent
CLI = Path(os.environ.get("JARDE_CLI", "/tmp/jarde-cli-ann-fd-root-final"))
EXPECTED_CLI_SHA256 = "f2fb24241c2692dff53060576e185eb282617a6881e81900140bb2eb62006544"


def run(args: list[str], log: Path) -> int:
    result = subprocess.run(args, capture_output=True, text=True, timeout=60)
    log.write_text(result.stdout + result.stderr)
    log.with_suffix(log.suffix + ".exit").write_text(f"{result.returncode}\n")
    return result.returncode


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> None:
    assert CLI.is_file() and sha(CLI) == EXPECTED_CLI_SHA256
    for name in ("classes-original", "classes-jadx", "classes-jarde", "jadx"):
        shutil.rmtree(HERE / name, ignore_errors=True)
    original = HERE / "classes-original"
    original.mkdir()
    sources = [HERE / "FloatDefaults.java", HERE / "FloatRunner.java"]
    assert run(["javac", "--release", "8", "-g:none", "-d", str(original),
                *(str(path) for path in sources)], HERE / "original-javac.log") == 0
    classes = sorted(original.glob("*.class"))
    assert len(classes) == 2
    hashes = {path.name: sha(path) for path in classes}
    for path in classes:
        assert run(["javap", "-v", "-c", "-p", str(path)], HERE / f"javap-{path.stem}.txt") == 0
    assert run(["java", "-Xverify:all", "-cp", str(original), "FloatRunner"],
               HERE / "original-run.txt") == 0

    jadx_dir = HERE / "jadx"
    assert run(["jadx", "-d", str(jadx_dir), *(str(path) for path in classes)],
               HERE / "jadx.log") == 0
    jadx_sources = sorted((jadx_dir / "sources").rglob("*.java"))
    assert len(jadx_sources) == 2
    jadx_classes = HERE / "classes-jadx"
    jadx_classes.mkdir()
    assert run(["javac", "--release", "8", "-g:none", "-d", str(jadx_classes),
                *(str(path) for path in jadx_sources)], HERE / "jadx-javac.log") == 0
    assert run(["java", "-Xverify:all", "-cp", str(jadx_classes), "defpackage.FloatRunner"],
               HERE / "jadx-run.txt") == 0

    for path in classes:
        args = [str(CLI), "class-source", "--input", str(path), "--class", path.stem,
                "--policy", "single-class", "--release", "8"]
        text = subprocess.run([*args, "--format", "text"], capture_output=True, text=True, timeout=60)
        (HERE / f"jarde-{path.stem}.java").write_text(text.stdout)
        (HERE / f"jarde-{path.stem}.log").write_text(text.stderr)
        assert text.returncode == 0
        report = subprocess.run([*args, "--format", "json", "--evidence", "all"],
                                capture_output=True, text=True, timeout=60)
        (HERE / f"jarde-{path.stem}.json").write_text(report.stdout)
        assert report.returncode == 0
    jarde_classes = HERE / "classes-jarde"
    jarde_classes.mkdir()
    jarde_sources = [HERE / "jarde-FloatDefaults.java", HERE / "jarde-FloatRunner.java"]
    jarde_javac = run(["javac", "--release", "8", "-g:none", "-d", str(jarde_classes),
                       *(str(path) for path in jarde_sources)], HERE / "jarde-javac.log")
    jarde_runtime = run(["java", "-Xverify:all", "-cp", str(jarde_classes), "FloatRunner"],
                        HERE / "jarde-run.txt")
    original_output = (HERE / "original-run.txt").read_text()
    jadx_output = (HERE / "jadx-run.txt").read_text()
    assert original_output == jadx_output == "80000000\n1\n7f800000\n7ff8000000000000\n"
    assert jarde_javac == 0 and jarde_runtime == 0
    assert (HERE / "jarde-run.txt").read_text() == original_output
    assert "default " in (HERE / "jarde-FloatDefaults.java").read_text()
    assert sha(CLI) == EXPECTED_CLI_SHA256
    (HERE / "summary.json").write_text(json.dumps({
        "cli_sha256": EXPECTED_CLI_SHA256,
        "class_sha256": hashes,
        "annotation_class_bytes": (original / "FloatDefaults.class").stat().st_size,
        "original_output": original_output.splitlines(),
        "jadx_output": jadx_output.splitlines(),
        "jarde_javac": jarde_javac,
        "jarde_runtime": jarde_runtime,
        "jarde_output": (HERE / "jarde-run.txt").read_text().splitlines(),
    }, indent=2) + "\n")


if __name__ == "__main__":
    main()
