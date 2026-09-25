#!/usr/bin/env python3
"""Rebuild a Java 8 class-annotation use and compare complete decompiled classes."""

from __future__ import annotations

import hashlib
import json
import os
import shutil
import subprocess
from pathlib import Path


HERE = Path(__file__).resolve().parent
OUT = HERE / "generated"
CLI = Path(os.environ.get("JARDE_CLI", "/tmp/jarde-cli-nested-root-final"))
CLI_SHA256 = "7f9dd55bb020d0308504a9484ad44b157b271f08b1b96c667e538f935c6a38d5"


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run(args: list[str], name: str) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(args, capture_output=True, text=True, timeout=90)
    (OUT / f"{name}.log").write_text(result.stdout + result.stderr)
    (OUT / f"{name}.exit").write_text(f"{result.returncode}\n")
    return result


def main() -> None:
    assert CLI.is_file() and sha(CLI) == CLI_SHA256
    shutil.rmtree(OUT, ignore_errors=True)
    OUT.mkdir()
    original = OUT / "original"
    original.mkdir()
    sources = [HERE / "Tagged.java", HERE / "TagRunner.java", HERE / "RetentionTagged.java"]
    assert run(["javac", "--release", "8", "-g:none", "-Xlint:-options", "-d", str(original),
                *(str(path) for path in sources)], "original-javac").returncode == 0
    class_files = sorted(original.glob("*.class"))
    assert [p.stem for p in class_files] == ["RetentionTagged", "TagRunner", "Tagged"]
    for class_file in class_files:
        assert run(["javap", "-v", "-c", "-p", str(class_file)],
                   f"javap-{class_file.stem}").returncode == 0
    assert run(["java", "-Xverify:all", "-cp", str(original), "TagRunner"],
               "original-run").returncode == 0

    jadx = OUT / "jadx"
    assert run(["jadx", "-d", str(jadx), *(str(path) for path in class_files)],
               "jadx").returncode == 0
    jadx_sources = sorted((jadx / "sources").rglob("*.java"))
    assert [p.stem for p in jadx_sources] == ["RetentionTagged", "TagRunner", "Tagged"]
    jadx_classes = OUT / "jadx-classes"
    jadx_classes.mkdir()
    assert run(["javac", "--release", "8", "-g:none", "-Xlint:-options", "-d", str(jadx_classes),
                *(str(path) for path in jadx_sources)], "jadx-javac").returncode == 0
    assert run(["java", "-Xverify:all", "-cp", str(jadx_classes), "defpackage.TagRunner"],
               "jadx-run").returncode == 0

    jarde_sources = []
    jarde_source_dir = OUT / "jarde-src"
    jarde_source_dir.mkdir()
    for class_file in class_files:
        base = [str(CLI), "class-source", "--input", str(class_file), "--class", class_file.stem,
                "--policy", "single-class", "--release", "8"]
        source = run([*base, "--format", "text"], f"jarde-{class_file.stem}-text")
        report = run([*base, "--format", "json", "--evidence", "all"],
                     f"jarde-{class_file.stem}-json")
        assert source.returncode == report.returncode == 0
        source_path = OUT / f"jarde-{class_file.stem}.java"
        source_path.write_text(source.stdout)
        (OUT / f"jarde-{class_file.stem}.json").write_text(report.stdout)
        compile_path = jarde_source_dir / f"{class_file.stem}.java"
        compile_path.write_text(source.stdout)
        jarde_sources.append(compile_path)
    jarde_classes = OUT / "jarde-classes"
    jarde_classes.mkdir()
    assert run(["javac", "--release", "8", "-g:none", "-Xlint:-options", "-d", str(jarde_classes),
                *(str(path) for path in jarde_sources)], "jarde-javac").returncode == 0
    assert run(["java", "-Xverify:all", "-cp", str(jarde_classes), "TagRunner"],
               "jarde-run").returncode == 0

    observed = {kind: (OUT / f"{kind}-run.log").read_text()
                for kind in ("original", "jadx", "jarde")}
    assert observed["original"] == observed["jadx"]
    assert observed["original"].startswith("true\n3\n@java.lang.annotation.Retention(RUNTIME)\n")
    assert observed["jarde"] == "false\n3\nnull\n"
    assert "RuntimeVisibleAnnotations:" in (OUT / "javap-Tagged.log").read_text()
    assert "@Deprecated" in next(p for p in jadx_sources if p.stem == "Tagged").read_text()
    assert "@Deprecated" not in (OUT / "jarde-Tagged.java").read_text()
    assert "@java.lang.annotation.Retention" not in (OUT / "jarde-RetentionTagged.java").read_text()
    assert sha(CLI) == CLI_SHA256
    (OUT / "summary.json").write_text(json.dumps({
        "cli_sha256": CLI_SHA256,
        "source_sha256": {p.name: sha(p) for p in sources},
        "class_sha256": {p.name: sha(p) for p in class_files},
        "class_bytes": {p.name: p.stat().st_size for p in class_files},
        "output": {name: value.splitlines() for name, value in observed.items()},
    }, indent=2) + "\n")


if __name__ == "__main__":
    main()
