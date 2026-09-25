#!/usr/bin/env python3
"""Rebuild and compare field, method and parameter annotation use with Java 8."""

from __future__ import annotations

import hashlib
import json
import os
import shutil
import subprocess
from pathlib import Path


HERE = Path(__file__).resolve().parent
OUT = HERE / "generated"
CLI = Path(os.environ.get("JARDE_CLI", "/tmp/jarde-cli-ann-fd-root-final"))
CLI_SHA256 = "f2fb24241c2692dff53060576e185eb282617a6881e81900140bb2eb62006544"


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run(args: list[str], log: Path) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(args, capture_output=True, text=True, timeout=90)
    log.write_text(result.stdout + result.stderr)
    log.with_suffix(log.suffix + ".exit").write_text(f"{result.returncode}\n")
    return result


def main() -> None:
    assert CLI.is_file() and sha(CLI) == CLI_SHA256
    shutil.rmtree(OUT, ignore_errors=True)
    OUT.mkdir()
    original = OUT / "original"
    original.mkdir()
    sources = [HERE / "MemberTagged.java", HERE / "MemberRunner.java"]
    assert run(["javac", "--release", "8", "-g:none", "-Xlint:-options", "-d", str(original),
                *(str(source) for source in sources)], OUT / "original-javac.log").returncode == 0
    classes = sorted(original.glob("*.class"))
    assert [path.stem for path in classes] == ["MemberRunner", "MemberTagged"]
    for path in classes:
        assert run(["javap", "-v", "-c", "-p", str(path)],
                   OUT / f"javap-{path.stem}.txt").returncode == 0
    assert run(["java", "-Xverify:all", "-cp", str(original), "MemberRunner"],
               OUT / "original-run.txt").returncode == 0

    jadx = OUT / "jadx"
    assert run(["jadx", "-d", str(jadx), *(str(path) for path in classes)],
               OUT / "jadx.log").returncode == 0
    jadx_sources = sorted((jadx / "sources").rglob("*.java"))
    assert [path.stem for path in jadx_sources] == ["MemberRunner", "MemberTagged"]
    jadx_classes = OUT / "jadx-classes"
    jadx_classes.mkdir()
    assert run(["javac", "--release", "8", "-g:none", "-Xlint:-options", "-d", str(jadx_classes),
                *(str(path) for path in jadx_sources)], OUT / "jadx-javac.log").returncode == 0
    assert run(["java", "-Xverify:all", "-cp", str(jadx_classes), "defpackage.MemberRunner"],
               OUT / "jadx-run.txt").returncode == 0

    jarde_source_dir = OUT / "jarde-src"
    jarde_source_dir.mkdir()
    for path in classes:
        base = [str(CLI), "class-source", "--input", str(path), "--class", path.stem,
                "--policy", "single-class", "--release", "8"]
        source = run([*base, "--format", "text"], OUT / f"jarde-{path.stem}-text.log")
        report = run([*base, "--format", "json", "--evidence", "all"],
                     OUT / f"jarde-{path.stem}-json.log")
        assert source.returncode == report.returncode == 0
        (OUT / f"jarde-{path.stem}.java").write_text(source.stdout)
        (OUT / f"jarde-{path.stem}.json").write_text(report.stdout)
        (jarde_source_dir / f"{path.stem}.java").write_text(source.stdout)
    jarde_classes = OUT / "jarde-classes"
    jarde_classes.mkdir()
    assert run(["javac", "--release", "8", "-g:none", "-Xlint:-options", "-d", str(jarde_classes),
                *(str(path) for path in sorted(jarde_source_dir.glob("*.java")))],
               OUT / "jarde-javac.log").returncode == 0
    assert run(["java", "-Xverify:all", "-cp", str(jarde_classes), "MemberRunner"],
               OUT / "jarde-run.txt").returncode == 0

    output = {name: (OUT / f"{name}-run.txt").read_text()
              for name in ("original", "jadx", "jarde")}
    assert output == {"original": "true\ntrue\n1\n5\n",
                      "jadx": "true\ntrue\n1\n5\n",
                      "jarde": "false\nfalse\n0\n5\n"}
    assert sha(CLI) == CLI_SHA256
    (OUT / "summary.json").write_text(json.dumps({
        "cli_sha256": CLI_SHA256,
        "source_sha256": {path.name: sha(path) for path in sources},
        "class_sha256": {path.name: sha(path) for path in classes},
        "class_bytes": {path.name: path.stat().st_size for path in classes},
        "output": {name: value.splitlines() for name, value in output.items()},
    }, indent=2) + "\n")


if __name__ == "__main__":
    main()
