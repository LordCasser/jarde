#!/usr/bin/env python3
"""Replay isolated DT-18 field, method-signature, null-return, and raw controls."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import zipfile


HERE = Path(__file__).resolve().parent
JADX_REPO = Path("/Users/lordcasser/workspace/testzone/jadx")
JADX = JADX_REPO / "jadx-cli/build/install/jadx/bin/jadx"
JADX_HEAD = "2fb1b16386941660fda07e9017285aec40fcb37f"
EXPECTED = (
    "field=java.util.List<java.lang.String>\n"
    "id-param=java.util.List<java.lang.String>\n"
    "id-return=java.util.List<java.lang.String>\n"
    "empty-return=java.util.List<java.lang.String>\n"
    "raw-field=interface java.util.List\n"
    "raw-param=interface java.util.List\n"
    "raw-return=interface java.util.List\n"
    "values=ok:true\n"
)
BASELINE_JARDE = EXPECTED.replace(
    "empty-return=java.util.List<java.lang.String>",
    "empty-return=interface java.util.List",
)


def run(*args: object) -> subprocess.CompletedProcess[str]:
    return subprocess.run([str(arg) for arg in args], text=True, capture_output=True)


def checked(result: subprocess.CompletedProcess[str], label: str) -> None:
    if result.returncode:
        raise SystemExit(f"{label}: exit {result.returncode}: {result.stderr[-3000:]}")


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def compile_sources(classes: Path, *sources: Path) -> None:
    classes.mkdir()
    checked(run("javac", "-J-Duser.language=en", "-J-Duser.country=US", "--release", "8",
                "-g:none", "-Xlint:-options", "-d", classes, *sources), "javac")


def verified_run(classes: Path, label: str) -> str:
    result = run("java", "-Xverify:all", "-cp", classes, "dt18.Runner")
    checked(result, label)
    return result.stdout


if len(sys.argv) != 3 or sys.argv[1] not in ("baseline", "fixed"):
    raise SystemExit("usage: replay.py baseline|fixed /absolute/path/to/jarde-cli")
mode, cli_arg = sys.argv[1:]
cli = Path(cli_arg)
if not cli.is_file() or not JADX.is_file():
    raise SystemExit("Jarde CLI or pinned JADX executable is absent")
head = run("git", "-C", JADX_REPO, "rev-parse", "HEAD")
status = run("git", "-C", JADX_REPO, "status", "--porcelain")
if head.returncode or head.stdout.strip() != JADX_HEAD or status.returncode or status.stdout.strip():
    raise SystemExit("JADX checkout must be clean at the frozen commit")
output = HERE / "outputs" / ("fixed" if mode == "fixed" else "")
output.mkdir(exist_ok=True)

with tempfile.TemporaryDirectory(prefix="jarde-dt18-audit-") as temporary:
    work = Path(temporary)
    original = work / "original"
    compile_sources(original, HERE / "GenericSlots.java", HERE / "Runner.java")
    original_run = verified_run(original, "original")
    if original_run != EXPECTED:
        raise SystemExit("original parameterized member runtime differs from frozen result")
    javap = run("javap", "-c", "-p", "-s", "-classpath", original,
                "dt18.GenericSlots")
    checked(javap, "original javap")
    (output / "original-javap.txt").write_text(javap.stdout)
    jar = work / "input.jar"
    with zipfile.ZipFile(jar, "w", zipfile.ZIP_DEFLATED) as archive:
        archive.write(original / "dt18/GenericSlots.class", "dt18/GenericSlots.class")

    jadx_root = work / "jadx"
    checked(run(JADX, "-d", jadx_root, jar), "JADX")
    jadx_source = jadx_root / "sources/dt18/GenericSlots.java"
    if not jadx_source.is_file():
        raise SystemExit("JADX did not produce the complete type")
    (output / "jadx-GenericSlots.java.txt").write_text(jadx_source.read_text())
    jadx_classes = work / "jadx-classes"
    compile_sources(jadx_classes, jadx_source, HERE / "Runner.java")
    jadx_run = verified_run(jadx_classes, "JADX")
    if jadx_run != EXPECTED:
        raise SystemExit("JADX parameterized member runtime differs from original")

    jarde = run(cli, "class-source", "--input", jar, "--class", "dt18/GenericSlots",
                "--policy", "plain-jar", "--release", "8", "--format", "text")
    checked(jarde, "Jarde class-source")
    jarde_source = work / "jarde/dt18/GenericSlots.java"
    jarde_source.parent.mkdir(parents=True)
    jarde_source.write_text(jarde.stdout)
    (output / "jarde-GenericSlots.java.txt").write_text(jarde.stdout)
    jarde_classes = work / "jarde-classes"
    compile_sources(jarde_classes, jarde_source, HERE / "Runner.java")
    jarde_run = verified_run(jarde_classes, "Jarde")
    (output / "jarde-run.log").write_text(jarde_run)
    expected_jarde = EXPECTED if mode == "fixed" else BASELINE_JARDE
    expected_refusals = 0 if mode == "fixed" else 1
    if (jarde_run != expected_jarde
            or jarde.stdout.count("generic Signature projection refused") != expected_refusals):
        raise SystemExit(f"{mode} Jarde generic null return/refusal changed")

    results = {
        "jadx_head": JADX_HEAD,
        "javac_version": run("javac", "-version").stdout.strip(),
        "source_sha256": {name: sha(HERE / name) for name in ("GenericSlots.java", "Runner.java")},
        "original_class_sha256": sha(original / "dt18/GenericSlots.class"),
        "output_sha256": {
            "jadx_source": sha(output / "jadx-GenericSlots.java.txt"),
            "jarde_source": sha(output / "jarde-GenericSlots.java.txt"),
            "jarde_verified_run": sha(output / "jarde-run.log"),
        },
        "original_verified_run": original_run,
        "jadx_verified_run": jadx_run,
        "jarde_verified_run": jarde_run,
    }
    (output / "results.json").write_text(json.dumps(results, ensure_ascii=False, indent=2) + "\n")
    print(f"DT-18 {mode} replay complete")
