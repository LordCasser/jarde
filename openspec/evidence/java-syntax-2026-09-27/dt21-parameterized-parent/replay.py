#!/usr/bin/env python3
"""Replay the direct DT-21 parameterized superclass boundary."""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
from zipfile import ZIP_DEFLATED, ZipFile, ZipInfo

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
JADX_ROOT = Path("/Users/lordcasser/workspace/testzone/jadx")
JADX = JADX_ROOT / "jadx-cli/build/install/jadx/bin/jadx"
JADX_HEAD = "2fb1b16386941660fda07e9017285aec40fcb37f"
OUT = HERE / "outputs"
EXPECTED = "dt21parent.Parent<java.lang.String>\n"


def run(*args: object, **kwargs: object) -> subprocess.CompletedProcess[str]:
    return subprocess.run([str(arg) for arg in args], text=True, capture_output=True, **kwargs)


def checked(*args: object, **kwargs: object) -> str:
    result = run(*args, **kwargs)
    if result.returncode:
        raise RuntimeError(f"{args[0]} exited {result.returncode}: {result.stderr[-3000:]}")
    return result.stdout


def require(condition: bool, message: str) -> None:
    if not condition:
        raise RuntimeError(message)


def compile_run(sources: list[Path], work: Path, label: str) -> dict[str, object]:
    classes = work / f"{label}-classes"
    classes.mkdir()
    result = run("javac", "-J-Duser.language=en", "-J-Duser.country=US",
                 "--release", "8", "-g:none", "-Xlint:-options", "-d", classes, *sources)
    if result.returncode:
        return {"compile_exit": result.returncode,
                "compile_stderr": result.stderr.replace(str(work), "<TMP>")}
    verified = run("java", "-Xverify:all", "-cp", classes, "dt21parent.Runner")
    return {"compile_exit": 0, "run_exit": verified.returncode,
            "run_stdout": verified.stdout, "run_stderr": verified.stderr}


head = checked("git", "rev-parse", "HEAD", cwd=JADX_ROOT).strip()
require(head == JADX_HEAD, f"JADX checkout changed: {head}")
require(not checked("git", "status", "--porcelain", cwd=JADX_ROOT).strip(),
        "pinned JADX checkout is not clean")
require(JADX.is_file(), "pinned JADX CLI is missing")
OUT.mkdir(exist_ok=True)

with tempfile.TemporaryDirectory(prefix="jarde-dt21-parent-") as temporary:
    work = Path(temporary)
    original = compile_run([HERE / "Child.java", HERE / "Runner.java"], work, "original")
    require(original == {"compile_exit": 0, "run_exit": 0,
                         "run_stdout": EXPECTED, "run_stderr": ""},
            f"original Java 8 verified result changed: {original}")
    class_files = sorted((work / "original-classes/dt21parent").glob("*.class"))
    class_files = [path for path in class_files if path.name != "Runner.class"]
    require([path.stem for path in class_files] == ["Child", "Parent"],
            "expected direct child and parent classfiles")
    for path in class_files:
        javap = checked("javap", "-v", "-p", "-classpath", work / "original-classes",
                        f"dt21parent/{path.stem}")
        javap = javap.replace(str(work), "<TMP>").replace(str(ROOT), "<REPO>")
        javap = re.sub(r"(?m)^  Last modified .*; size (\d+) bytes$",
                        r"  Last modified <NORMALIZED>; size \1 bytes", javap)
        (OUT / f"javap-{path.stem}.txt").write_text(javap)

    archive = work / "input.jar"
    with ZipFile(archive, "w", ZIP_DEFLATED) as jar:
        for path in class_files:
            entry = ZipInfo(f"dt21parent/{path.name}", date_time=(2000, 1, 1, 0, 0, 0))
            entry.compress_type = ZIP_DEFLATED
            jar.writestr(entry, path.read_bytes())
    jadx_dir = work / "jadx"
    checked(JADX, "-d", jadx_dir, archive)
    jadx_sources = sorted(jadx_dir.rglob("*.java"))
    require(len(jadx_sources) == 2, "JADX did not emit two complete source files")
    for path in jadx_sources:
        (OUT / f"jadx-{path.stem}.java.txt").write_text(path.read_text())
    jadx = compile_run(jadx_sources + [HERE / "Runner.java"], work, "jadx")
    require(jadx.get("run_stdout") == EXPECTED and jadx.get("run_exit") == 0,
            f"JADX Java 8 verified result changed: {jadx}")

    cli = os.environ.get("JARDE_CLI")
    if cli is None:
        env = os.environ.copy()
        env.update(CARGO_TARGET_DIR=str(work / "cargo-target"), CARGO_INCREMENTAL="0",
                   CARGO_BUILD_JOBS="2")
        build = run("cargo", "build", "-p", "jarde-cli", "--locked", cwd=ROOT, env=env)
        require(build.returncode == 0, build.stderr[-3000:])
        cli = str(work / "cargo-target/debug/jarde-cli")
    jarde_src = work / "jarde-src/dt21parent"
    jarde_src.mkdir(parents=True)
    for path in class_files:
        source = checked(cli, "class-source", "--input", archive,
                         "--class", f"dt21parent/{path.stem}", "--policy", "plain-jar",
                         "--release", "8", "--format", "text")
        (OUT / f"jarde-{path.stem}.java.txt").write_text(source)
        (jarde_src / f"{path.stem}.java").write_text(source)
    jarde = compile_run(sorted(jarde_src.glob("*.java")) + [HERE / "Runner.java"], work, "jarde")
    require(jarde.get("run_stdout") == "class dt21parent.Parent\n" and
            jarde.get("run_exit") == 0,
            f"baseline Jarde raw parent boundary changed: {jarde}")
    require("class Child extends dt21parent.Parent {" in
            (OUT / "jarde-Child.java.txt").read_text(),
            "baseline Jarde raw parent declaration changed")

    results = {
        "jadx_head": head,
        "javac": checked("javac", "-version").strip(),
        "source_sha256": {path.name: hashlib.sha256(path.read_bytes()).hexdigest()
                          for path in (HERE / "Child.java", HERE / "Runner.java")},
        "class_sha256": {path.name: hashlib.sha256(path.read_bytes()).hexdigest()
                         for path in class_files},
        "original": original,
        "jadx": jadx,
        "jarde": jarde,
        "assertions": {
            "original_and_jadx_complete_java8_sources_verify_and_preserve_parent_type": True,
            "jarde_complete_source_compiles_but_erases_parent_type": True,
        },
    }
    (OUT / "results.json").write_text(json.dumps(results, indent=2, sort_keys=True) + "\n")
    print(json.dumps(results["assertions"], sort_keys=True))
