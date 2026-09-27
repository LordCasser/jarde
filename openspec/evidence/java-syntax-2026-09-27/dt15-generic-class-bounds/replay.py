#!/usr/bin/env python3
"""Replay the bounded DT-15 class/interface type-parameter comparison."""
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
NAMES = ("Bounds", "Contract", "Plain")
EXPECTED = "T:java.lang.Number:java.lang.Comparable<T>\nX:java.lang.Number\n0:true\n"


def run(*args: object) -> subprocess.CompletedProcess[str]:
    return subprocess.run([str(arg) for arg in args], text=True, capture_output=True)


def require(result: subprocess.CompletedProcess[str], label: str) -> None:
    if result.returncode:
        raise SystemExit(f"{label}: exit {result.returncode}: {result.stderr[-3000:]}")


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def compile_and_run(sources: list[Path], classes: Path, label: str) -> dict[str, object]:
    classes.mkdir()
    require(run("javac", "-J-Duser.language=en", "-J-Duser.country=US", "--release", "8",
                "-g:none", "-Xlint:-options", "-d", classes, *sources), label + " javac")
    result = run("java", "-Xverify:all", "-cp", classes, "dt15.Runner")
    require(result, label + " java")
    if result.stdout != EXPECTED:
        raise SystemExit(f"{label}: unexpected runtime {result.stdout!r}")
    return {"compile_exit": 0, "verified_run_exit": 0, "stdout": result.stdout}


if len(sys.argv) != 2 or not Path(sys.argv[1]).is_file():
    raise SystemExit("usage: replay.py /absolute/path/to/jarde-cli")
if run("git", "-C", JADX_REPO, "rev-parse", "HEAD").stdout.strip() != JADX_HEAD:
    raise SystemExit("JADX checkout HEAD differs from frozen baseline")
if run("git", "-C", JADX_REPO, "status", "--porcelain").stdout.strip():
    raise SystemExit("JADX checkout is not clean")
if not JADX.is_file():
    raise SystemExit("JADX executable is absent")

with tempfile.TemporaryDirectory(prefix="jarde-dt15-audit-") as temporary:
    work = Path(temporary)
    original = work / "original"
    original_result = compile_and_run(
        [*(HERE / f"{name}.java" for name in NAMES), HERE / "Runner.java"], original,
        "original")
    jar = work / "input.jar"
    with zipfile.ZipFile(jar, "w", zipfile.ZIP_DEFLATED) as archive:
        for path in sorted((original / "dt15").glob("*.class")):
            if path.name != "Runner.class":
                archive.write(path, f"dt15/{path.name}")

    jadx_root = work / "jadx"
    require(run(JADX, "-d", jadx_root, jar), "JADX")
    jadx_files = sorted(jadx_root.rglob("*.java"))
    if len(jadx_files) != len(NAMES):
        raise SystemExit("JADX source set is incomplete")
    jadx_result = compile_and_run([*jadx_files, HERE / "Runner.java"], work / "jadx-classes", "JADX")

    jarde_root = work / "jarde"
    jarde_files = []
    outputs = HERE / "outputs"
    outputs.mkdir(exist_ok=True)
    for name in NAMES:
        source = run(sys.argv[1], "class-source", "--input", jar, "--class", f"dt15/{name}",
                     "--policy", "plain-jar", "--release", "8", "--format", "text")
        require(source, f"Jarde {name}")
        path = jarde_root / "dt15" / f"{name}.java"
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(source.stdout)
        (outputs / f"jarde-{name}.java.txt").write_text(source.stdout)
        jarde_files.append(path)
    jarde_result = compile_and_run([*jarde_files, HERE / "Runner.java"], work / "jarde-classes", "Jarde")
    for source in jadx_files:
        (outputs / f"jadx-{source.name}.txt").write_text(source.read_text())

    bounded = jarde_files[0].read_text()
    contract = jarde_files[1].read_text()
    plain = jarde_files[2].read_text()
    if ("Bounds<T extends java.lang.Number & java.lang.Comparable<T>>" not in bounded
            or "interface Contract<X extends java.lang.Number>" not in contract
            or "class Plain<" in plain):
        raise SystemExit("Jarde class/interface generic header or non-generic control differs")
    results = {
        "jadx_head": JADX_HEAD,
        "javac": run("javac", "-version").stdout.strip(),
        "source_sha256": {p.name: sha(p) for p in [*(HERE / f"{n}.java" for n in NAMES), HERE / "Runner.java"]},
        "class_sha256": {p.name: sha(p) for p in sorted((original / "dt15").glob("*.class"))},
        "outputs_sha256": {p.name: sha(p) for p in sorted(outputs.iterdir())},
        "original": original_result,
        "jadx": jadx_result,
        "jarde": jarde_result,
        "scope": "top-level class and interface bounds only; no nested generic member or generic method claim",
    }
    (HERE / "results.json").write_text(json.dumps(results, indent=2, ensure_ascii=False) + "\n")
    print("DT-15 top-level generic bounds: original/JADX/Jarde Java 8 compile and verified run match")
