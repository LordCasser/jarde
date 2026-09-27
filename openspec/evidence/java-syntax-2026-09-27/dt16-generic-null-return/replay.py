#!/usr/bin/env python3
"""Replay the DT-16 no-argument generic null-return slice with Java 8."""
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
EXPECTED_JADX_HEAD = "2fb1b16386941660fda07e9017285aec40fcb37f"
EXPECTED = "true:T:java.lang.Number:T\n"


def run(*args: object) -> subprocess.CompletedProcess[str]:
    return subprocess.run([str(arg) for arg in args], capture_output=True, text=True)


def checked(result: subprocess.CompletedProcess[str], label: str) -> None:
    if result.returncode:
        raise SystemExit(f"{label}: exit {result.returncode}: {result.stderr[-3000:]}")


def normalize(text: str, temp: Path) -> str:
    return text.replace(str(temp), "<TEMP>").replace(str(ROOT), "<REPO>")


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def compile_sources(classes: Path, *sources: Path) -> subprocess.CompletedProcess[str]:
    classes.mkdir()
    return run("javac", "-J-Duser.language=en", "-J-Duser.country=US", "--release", "8",
               "-g:none", "-Xlint:-options", "-d", classes, *sources)


if len(sys.argv) != 3 or sys.argv[1] not in ("baseline", "fixed"):
    raise SystemExit("usage: replay.py baseline|fixed /absolute/path/to/jarde-cli")
mode, cli = sys.argv[1:]
if not Path(cli).is_file() or not JADX.is_file():
    raise SystemExit("Jarde CLI or pinned JADX executable is absent")
head = run("git", "-C", JADX_REPO, "rev-parse", "HEAD")
status = run("git", "-C", JADX_REPO, "status", "--porcelain")
if head.returncode or head.stdout.strip() != EXPECTED_JADX_HEAD or status.returncode or status.stdout.strip():
    raise SystemExit("JADX checkout must be clean at the frozen commit")
output = HERE / ("outputs" if mode == "baseline" else "fixed")
output.mkdir(exist_ok=True)

with tempfile.TemporaryDirectory(prefix="jarde-dt16-audit-") as temporary:
    work = Path(temporary)
    original = work / "original"
    checked(compile_sources(original, HERE / "NullResult.java", HERE / "Runner.java"), "original javac")
    original_run = run("java", "-Xverify:all", "-cp", original, "dt16.Runner")
    checked(original_run, "original java")
    if original_run.stdout != EXPECTED:
        raise SystemExit("original runtime differs from frozen result")
    javap = run("javap", "-c", "-p", "-s", "-classpath", original, "dt16.NullResult")
    checked(javap, "original javap")
    if ("descriptor: ()Ljava/lang/Number;" not in javap.stdout
            or "0: aconst_null" not in javap.stdout or "1: areturn" not in javap.stdout):
        raise SystemExit("original generic method is not the frozen two-instruction shape")
    (output / "original-javap.txt").write_text(javap.stdout)
    jar = work / "input.jar"
    with zipfile.ZipFile(jar, "w", zipfile.ZIP_DEFLATED) as archive:
        archive.write(original / "dt16/NullResult.class", "dt16/NullResult.class")

    jadx_root = work / "jadx"
    checked(run(JADX, "-d", jadx_root, jar), "JADX")
    jadx_source = jadx_root / "sources/dt16/NullResult.java"
    if not jadx_source.is_file():
        raise SystemExit("JADX did not produce the expected complete type")
    (output / "jadx-NullResult.java.txt").write_text(jadx_source.read_text())
    jadx_classes = work / "jadx-classes"
    checked(compile_sources(jadx_classes, jadx_source, HERE / "Runner.java"), "JADX javac")
    jadx_run = run("java", "-Xverify:all", "-cp", jadx_classes, "dt16.Runner")
    checked(jadx_run, "JADX java")
    if jadx_run.stdout != EXPECTED:
        raise SystemExit("JADX runtime differs from original")

    jarde = run(cli, "class-source", "--input", jar, "--class", "dt16/NullResult",
                "--policy", "plain-jar", "--release", "8", "--format", "text")
    checked(jarde, "Jarde class-source")
    jarde_source = work / "jarde/dt16/NullResult.java"
    jarde_source.parent.mkdir(parents=True)
    jarde_source.write_text(jarde.stdout)
    (output / "jarde-NullResult.java.txt").write_text(jarde.stdout)
    jarde_classes = work / "jarde-classes"
    jarde_compile = compile_sources(jarde_classes, jarde_source, HERE / "Runner.java")
    diagnostic = normalize(jarde_compile.stderr, work)
    (output / "jarde-javac.log").write_text(diagnostic + f"exit={jarde_compile.returncode}\n")
    if mode == "baseline":
        if (jarde_compile.returncode == 0 or "incompatible types: Number cannot be converted to Integer" not in diagnostic
                or "generic Signature projection refused" not in jarde.stdout):
            raise SystemExit("baseline generic-method refusal or consumer diagnostic changed")
        jarde_result = {"compile_exit": jarde_compile.returncode, "diagnostic": diagnostic,
                        "verified_run": None}
    else:
        checked(jarde_compile, "fixed Jarde javac")
        jarde_run = run("java", "-Xverify:all", "-cp", jarde_classes, "dt16.Runner")
        checked(jarde_run, "fixed Jarde java")
        if jarde_run.stdout != EXPECTED:
            raise SystemExit("fixed Jarde runtime differs from original")
        jarde_result = {"compile_exit": 0, "verified_run": jarde_run.stdout}

    results = {
        "mode": mode,
        "jadx_head": EXPECTED_JADX_HEAD,
        "javac": run("javac", "-version").stdout.strip(),
        "source_sha256": {name: sha(HERE / name) for name in ("NullResult.java", "Runner.java")},
        "original_class_sha256": sha(original / "dt16/NullResult.class"),
        "original_verified_run": original_run.stdout,
        "jadx_verified_run": jadx_run.stdout,
        "jarde": jarde_result,
    }
    (output / "results.json").write_text(json.dumps(results, ensure_ascii=False, indent=2) + "\n")
    print(f"DT-16 {mode} replay complete")
