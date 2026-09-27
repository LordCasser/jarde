#!/usr/bin/env python3
"""Replay the fixed DT-17 wildcard parameter forms against Java 8 output."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import zipfile


HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
JADX_REPO = Path("/Users/lordcasser/workspace/testzone/jadx")
JADX = JADX_REPO / "jadx-cli/build/install/jadx/bin/jadx"
JADX_HEAD = "2fb1b16386941660fda07e9017285aec40fcb37f"
NAMES = ("any", "ext", "sup", "extArray", "supArray", "raw")
EXPECTED = (
    "any=java.util.List<?>\n"
    "ext=java.util.List<? extends java.lang.Number>\n"
    "sup=java.util.List<? super java.lang.String>\n"
    "extArray=java.util.List<? extends byte[]>\n"
    "supArray=java.util.List<? super int[]>\n"
    "raw=java.util.List\n"
)
BASELINE_JARDE = "".join(f"{name}=java.util.List\n" for name in NAMES)


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
    result = run("java", "-Xverify:all", "-cp", classes, "dt17.Runner")
    checked(result, label)
    return result.stdout


if len(sys.argv) != 3 or sys.argv[1] not in ("baseline", "fixed"):
    raise SystemExit("usage: replay.py baseline|fixed /absolute/path/to/jarde-cli")
mode, cli = sys.argv[1:]
if not Path(cli).is_file() or not JADX.is_file():
    raise SystemExit("Jarde CLI or pinned JADX executable is absent")
head = run("git", "-C", JADX_REPO, "rev-parse", "HEAD")
status = run("git", "-C", JADX_REPO, "status", "--porcelain")
if head.returncode or head.stdout.strip() != JADX_HEAD or status.returncode or status.stdout.strip():
    raise SystemExit("JADX checkout must be clean at the frozen commit")
output = HERE / ("outputs" if mode == "baseline" else "fixed")
output.mkdir(exist_ok=True)

with tempfile.TemporaryDirectory(prefix="jarde-dt17-audit-") as temporary:
    work = Path(temporary)
    original = work / "original"
    compile_sources(original, HERE / "Wildcards.java", HERE / "Runner.java")
    original_run = verified_run(original, "original")
    if original_run != EXPECTED:
        raise SystemExit("original wildcard runtime differs from frozen result")
    javap = run("javap", "-c", "-p", "-s", "-classpath", original, "dt17.Wildcards")
    checked(javap, "original javap")
    (output / "original-javap.txt").write_text(javap.stdout)
    jar = work / "input.jar"
    with zipfile.ZipFile(jar, "w", zipfile.ZIP_DEFLATED) as archive:
        archive.write(original / "dt17/Wildcards.class", "dt17/Wildcards.class")

    jadx_root = work / "jadx"
    checked(run(JADX, "-d", jadx_root, jar), "JADX")
    jadx_source = jadx_root / "sources/dt17/Wildcards.java"
    if not jadx_source.is_file():
        raise SystemExit("JADX did not produce the complete type")
    (output / "jadx-Wildcards.java.txt").write_text(jadx_source.read_text())
    jadx_classes = work / "jadx-classes"
    compile_sources(jadx_classes, jadx_source, HERE / "Runner.java")
    jadx_run = verified_run(jadx_classes, "JADX")
    if jadx_run != EXPECTED:
        raise SystemExit("JADX wildcard runtime differs from original")

    jarde = run(cli, "class-source", "--input", jar, "--class", "dt17/Wildcards",
                "--policy", "plain-jar", "--release", "8", "--format", "text")
    checked(jarde, "Jarde class-source")
    jarde_source = work / "jarde/dt17/Wildcards.java"
    jarde_source.parent.mkdir(parents=True)
    jarde_source.write_text(jarde.stdout)
    (output / "jarde-Wildcards.java.txt").write_text(jarde.stdout)
    jarde_classes = work / "jarde-classes"
    compile_sources(jarde_classes, jarde_source, HERE / "Runner.java")
    jarde_run = verified_run(jarde_classes, "Jarde")
    (output / "jarde-run.log").write_text(jarde_run)
    if mode == "baseline":
        if (jarde_run != BASELINE_JARDE
                or jarde.stdout.count("generic Signature projection refused") != 5):
            raise SystemExit("baseline Jarde wildcard refusal or erasure output changed")
    elif jarde_run != EXPECTED:
        raise SystemExit("fixed Jarde wildcard runtime differs from original")

    results = {
        "mode": mode,
        "jadx_head": JADX_HEAD,
        "javac": run("javac", "-version").stdout.strip(),
        "source_sha256": {name: sha(HERE / name) for name in ("Wildcards.java", "Runner.java")},
        "original_class_sha256": sha(original / "dt17/Wildcards.class"),
        "original_verified_run": original_run,
        "jadx_verified_run": jadx_run,
        "jarde_verified_run": jarde_run,
    }
    (output / "results.json").write_text(json.dumps(results, ensure_ascii=False, indent=2) + "\n")
    print(f"DT-17 {mode} replay complete")
