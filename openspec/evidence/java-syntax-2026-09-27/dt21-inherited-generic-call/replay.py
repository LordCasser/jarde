#!/usr/bin/env python3
"""Replay one DT-21 generic superclass and inherited-call/bridge boundary."""
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


def normalized(text: str, work: Path) -> str:
    return text.replace(str(work), "<TMP>").replace(str(ROOT), "<REPO>")


def compile_run(sources: list[Path], work: Path, label: str) -> dict[str, object]:
    classes = work / f"{label}-classes"
    classes.mkdir()
    result = run("javac", "-J-Duser.language=en", "-J-Duser.country=US",
                 "--release", "8", "-g:none", "-Xlint:-options", "-d", classes, *sources)
    output: dict[str, object] = {"compile_exit": result.returncode}
    if result.returncode:
        output["compile_stderr"] = normalized(result.stderr, work)
        return output
    verified = run("java", "-Xverify:all", "-cp", classes, "dt21.Runner")
    output.update(run_exit=verified.returncode, run_stdout=verified.stdout,
                  run_stderr=normalized(verified.stderr, work))
    return output


head = checked("git", "rev-parse", "HEAD", cwd=JADX_ROOT).strip()
require(head == JADX_HEAD, f"JADX checkout changed: {head}")
require(not checked("git", "status", "--porcelain", cwd=JADX_ROOT).strip(),
        "pinned JADX checkout is not clean")
require(JADX.is_file(), "pinned JADX CLI is missing")
OUT.mkdir(exist_ok=True)

with tempfile.TemporaryDirectory(prefix="jarde-dt21-") as temporary:
    work = Path(temporary)
    original = compile_run([HERE / "Hierarchy.java", HERE / "Runner.java"], work, "original")
    require(original == {"compile_exit": 0, "run_exit": 0,
                         "run_stdout": "true\ndt21.C3<java.lang.String>\n", "run_stderr": ""},
            f"original Java 8 verified behavior changed: {original}")
    original_classes = work / "original-classes/dt21"
    class_files = sorted(original_classes.glob("*.class"))
    class_files = [path for path in class_files if path.name != "Runner.class"]
    require([path.stem for path in class_files] == ["C1", "C2", "C3", "Hierarchy"],
            "expected four physical hierarchy classes")
    for path in class_files:
        javap = checked("javap", "-v", "-p", "-classpath", work / "original-classes",
                        f"dt21/{path.stem}")
        javap = re.sub(r"(?m)^  Last modified .*; size (\d+) bytes$",
                        r"  Last modified <NORMALIZED>; size \1 bytes", normalized(javap, work))
        (OUT / f"javap-{path.stem}.txt").write_text(javap)

    archive = work / "input.jar"
    with ZipFile(archive, "w", ZIP_DEFLATED) as jar:
        for path in class_files:
            entry = ZipInfo(f"dt21/{path.name}", date_time=(2000, 1, 1, 0, 0, 0))
            entry.compress_type = ZIP_DEFLATED
            jar.writestr(entry, path.read_bytes())

    jadx_dir = work / "jadx"
    checked(JADX, "-d", jadx_dir, archive)
    jadx_sources = sorted(jadx_dir.rglob("*.java"))
    require(len(jadx_sources) == 4, "JADX did not emit the four-class source set")
    for path in jadx_sources:
        (OUT / f"jadx-{path.stem}.java.txt").write_text(path.read_text())
    jadx = compile_run(jadx_sources + [HERE / "Runner.java"], work, "jadx")
    require(jadx["compile_exit"] != 0 and
            "public /* bridge */ /* synthetic */ Object call()" in
            (OUT / "jadx-Hierarchy.java.txt").read_text(),
            "pinned JADX bridge recompilation boundary changed")

    cli = os.environ.get("JARDE_CLI")
    if cli is None:
        env = os.environ.copy()
        env.update(CARGO_TARGET_DIR=str(work / "cargo-target"), CARGO_INCREMENTAL="0",
                   CARGO_BUILD_JOBS="2")
        build = run("cargo", "build", "-p", "jarde-cli", "--locked", cwd=ROOT, env=env)
        require(build.returncode == 0, build.stderr[-3000:])
        cli = str(work / "cargo-target/debug/jarde-cli")
    jarde_src = work / "jarde-src/dt21"
    jarde_src.mkdir(parents=True)
    for path in class_files:
        source = checked(cli, "class-source", "--input", archive, "--class", f"dt21/{path.stem}",
                         "--policy", "plain-jar", "--release", "8", "--format", "text")
        (OUT / f"jarde-{path.stem}.java.txt").write_text(source)
        (jarde_src / f"{path.stem}.java").write_text(source)
    jarde = compile_run(sorted(jarde_src.glob("*.java")) + [HERE / "Runner.java"], work, "jarde")
    require(jarde.get("run_exit") == 0 and
            jarde.get("run_stdout") == "true\nclass dt21.C3\n",
            f"baseline Jarde generic parent behavior changed: {jarde}")
    require("class C2 extends dt21.C1" in (OUT / "jarde-C2.java.txt").read_text() and
            "class C3 extends dt21.C2" in (OUT / "jarde-C3.java.txt").read_text() and
            "class Hierarchy extends dt21.C3" in (OUT / "jarde-Hierarchy.java.txt").read_text(),
            "baseline Jarde raw generic parent boundary changed")

    results = {
        "jadx_head": head,
        "javac": checked("javac", "-version").strip(),
        "source_sha256": {path.name: hashlib.sha256(path.read_bytes()).hexdigest()
                          for path in (HERE / "Hierarchy.java", HERE / "Runner.java")},
        "class_sha256": {path.name: hashlib.sha256(path.read_bytes()).hexdigest()
                         for path in class_files},
        "original": original,
        "jadx": jadx,
        "jarde": jarde,
        "assertions": {
            "original_complete_java8_sources_verify_and_retain_generic_parent": True,
            "pinned_jadx_bridge_source_fails_java8_recompilation": True,
            "jarde_source_compiles_but_erases_parameterized_parent": True,
        },
    }
    (OUT / "results.json").write_text(json.dumps(results, indent=2, sort_keys=True) + "\n")
    print(json.dumps(results["assertions"], sort_keys=True))
