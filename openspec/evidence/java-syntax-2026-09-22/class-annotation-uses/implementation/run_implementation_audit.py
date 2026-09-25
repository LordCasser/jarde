#!/usr/bin/env python3
"""Replay the complete visible and invisible annotation cases with one frozen Jarde CLI."""

from __future__ import annotations

import hashlib
import json
import os
import shutil
import subprocess
from pathlib import Path


HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]
BASE = ROOT / "openspec/evidence/java-syntax-2026-09-22/class-annotation-uses"
BOUNDARIES = BASE / "boundaries/generated"
SOURCES = ROOT / "tests/fixtures/class-annotation-uses/src"
OUT = HERE / "generated"
CLI = Path(os.environ.get("JARDE_CLI", "/tmp/jarde-cli-class-ann-impl"))


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run(args: list[str], name: str) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(args, capture_output=True, text=True, timeout=90)
    (OUT / f"{name}.log").write_text(result.stdout + result.stderr)
    (OUT / f"{name}.exit").write_text(f"{result.returncode}\n")
    return result


def compile_java(output: Path, sources: list[Path], name: str) -> None:
    output.mkdir(parents=True, exist_ok=True)
    result = run(["javac", "--release", "8", "-g:none", "-Xlint:-options", "-d", str(output),
                  *(str(path) for path in sources)], name)
    assert result.returncode == 0, result.stderr


def jarde_sources(class_files: list[Path], output: Path, prefix: str) -> list[Path]:
    output.mkdir(parents=True, exist_ok=True)
    written = []
    for class_file in class_files:
        base = [str(CLI), "class-source", "--input", str(class_file), "--class", class_file.stem,
                "--policy", "single-class", "--release", "8"]
        source = run([*base, "--format", "text"], f"{prefix}-{class_file.stem}-text")
        report = run([*base, "--format", "json", "--evidence", "all"],
                     f"{prefix}-{class_file.stem}-json")
        assert source.returncode == report.returncode == 0
        source_path = output / f"{class_file.stem}.java"
        source_path.write_text(source.stdout)
        (OUT / f"{prefix}-{class_file.stem}.json").write_text(report.stdout)
        written.append(source_path)
    return written


def replay_base() -> dict[str, object]:
    source_names = ["Tagged.java", "RetentionTagged.java", "TagRunner.java"]
    sources = [BASE / name for name in source_names]
    original = OUT / "base-original-classes"
    compile_java(original, sources, "base-original-javac")
    class_files = sorted(original.glob("*.class"))
    for class_file in class_files:
        assert run(["javap", "-v", "-c", "-p", str(class_file)],
                   f"base-original-javap-{class_file.stem}").returncode == 0
    original_run = run(["java", "-Xverify:all", "-cp", str(original), "TagRunner"], "base-original-run")
    assert original_run.returncode == 0

    jadx_dir = OUT / "base-jadx"
    jadx = run(["jadx", "-d", str(jadx_dir), *(str(path) for path in class_files)], "base-jadx")
    assert jadx.returncode == 0
    jadx_sources = sorted((jadx_dir / "sources").rglob("*.java"))
    jadx_classes = OUT / "base-jadx-classes"
    compile_java(jadx_classes, jadx_sources, "base-jadx-javac")
    jadx_run = run(["java", "-Xverify:all", "-cp", str(jadx_classes), "defpackage.TagRunner"],
                   "base-jadx-run")
    assert jadx_run.returncode == 0

    jarde_source_dir = OUT / "base-jarde-src"
    jarde_source_files = jarde_sources(class_files, jarde_source_dir, "base-jarde")
    jarde_classes = OUT / "base-jarde-classes"
    compile_java(jarde_classes, jarde_source_files, "base-jarde-javac")
    jarde_run = run(["java", "-Xverify:all", "-cp", str(jarde_classes), "TagRunner"], "base-jarde-run")
    assert jarde_run.returncode == 0

    outputs = {name: (OUT / f"base-{name}-run.log").read_text()
               for name in ("original", "jadx", "jarde")}
    assert outputs["original"] == outputs["jadx"] == outputs["jarde"]
    assert outputs["original"].startswith("true\n3\n@java.lang.annotation.Retention(RUNTIME)\n")
    return {
        "source_sha256": {path.name: sha(path) for path in sources},
        "class_sha256": {path.name: sha(path) for path in class_files},
        "class_bytes": {path.name: path.stat().st_size for path in class_files},
        "outputs": {name: value.splitlines() for name, value in outputs.items()},
    }


def replay_invisible() -> dict[str, object]:
    sources = sorted(SOURCES.glob("*.java"))
    original = OUT / "invisible-original-classes"
    compile_java(original, sources, "invisible-original-javac")
    class_files = sorted(
        path for path in original.glob("*.class") if not path.stem.startswith("MemberPlacement")
    )
    original_run = run(["java", "-Xverify:all", "-cp", str(original), "BoundaryRunner"],
                       "invisible-original-run")
    assert original_run.returncode == 0

    jadx_dir = OUT / "invisible-jadx"
    jadx = run(["jadx", "-d", str(jadx_dir), *(str(path) for path in class_files)], "invisible-jadx")
    assert jadx.returncode == 0
    jadx_sources = sorted((jadx_dir / "sources").rglob("*.java"))
    jadx_classes = OUT / "invisible-jadx-classes"
    compile_java(jadx_classes, jadx_sources, "invisible-jadx-javac")
    jadx_run = run(["java", "-Xverify:all", "-cp", str(jadx_classes), "defpackage.BoundaryRunner"],
                   "invisible-jadx-run")
    assert jadx_run.returncode == 0

    jarde_source_dir = OUT / "invisible-jarde-src"
    jarde_source_files = jarde_sources(class_files, jarde_source_dir, "invisible-jarde")
    jarde_classes = OUT / "invisible-jarde-classes"
    compile_java(jarde_classes, jarde_source_files, "invisible-jarde-javac")
    jarde_run = run(["java", "-Xverify:all", "-cp", str(jarde_classes), "BoundaryRunner"],
                    "invisible-jarde-run")
    assert jarde_run.returncode == 0
    outputs = {name: (OUT / f"invisible-{name}-run.log").read_text()
               for name in ("original", "jadx", "jarde")}
    assert outputs["original"] == outputs["jadx"] == outputs["jarde"] == "0\nnull\n"

    original_hidden = original / "HiddenTarget.class"
    jarde_hidden = jarde_classes / "HiddenTarget.class"
    original_javap = run(["javap", "-v", "-c", "-p", str(original_hidden)], "invisible-original-javap")
    jarde_javap = run(["javap", "-v", "-c", "-p", str(jarde_hidden)], "invisible-jarde-javap")
    assert original_javap.returncode == jarde_javap.returncode == 0
    for output in (original_javap.stdout, jarde_javap.stdout):
        assert "RuntimeInvisibleAnnotations:" in output
        assert "HiddenTag(" in output and "value=5" in output

    boundary_inputs = {
        name: BOUNDARIES / name
        for name in ("DuplicateEntries.class", "DamagedValueTag.class", "UnspellableType.class")
    }
    boundary_reports: dict[str, object] = {}
    for filename, path in boundary_inputs.items():
        cls = "HiddenTarget"
        base = [str(CLI), "class-source", "--input", str(path), "--class", cls,
                "--policy", "single-class", "--release", "8"]
        text_result = run([*base, "--format", "text"], f"boundary-{path.stem}-text")
        json_result = run([*base, "--format", "json", "--evidence", "all"],
                          f"boundary-{path.stem}-json")
        expected_exit = 4 if filename == "DamagedValueTag.class" else 0
        assert text_result.returncode == json_result.returncode == expected_exit
        (OUT / f"boundary-{path.stem}.txt").write_text(text_result.stdout)
        report = json.loads(json_result.stdout)
        boundary_reports[filename] = {
            "text": text_result.stdout,
            "execution": report.get("execution"),
            "diagnostics": report.get("diagnostics"),
            "declaration": report.get("declaration"),
        }

    hashes = {path.name: sha(path) for path in class_files}
    return {
        "source_sha256": {path.name: sha(path) for path in sources},
        "class_sha256": hashes,
        "class_bytes": {path.name: path.stat().st_size for path in class_files},
        "outputs": {name: value.splitlines() for name, value in outputs.items()},
        "javap_runtime_invisible_same_type_and_value": True,
        "boundary_reports": boundary_reports,
    }


def main() -> None:
    assert CLI.is_file()
    cli_sha = sha(CLI)
    shutil.rmtree(OUT, ignore_errors=True)
    OUT.mkdir()
    base = replay_base()
    invisible = replay_invisible()
    (OUT / "summary.json").write_text(json.dumps({
        "cli_path": str(CLI),
        "cli_sha256": cli_sha,
        "java_version": run(["java", "-version"], "java-version").stderr.splitlines(),
        "base_visible_and_reflection": base,
        "class_retention_and_boundaries": invisible,
    }, indent=2) + "\n")


if __name__ == "__main__":
    main()
