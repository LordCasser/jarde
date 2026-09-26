#!/usr/bin/env python3
"""Recompile one CF-01 eager-boolean counterexample against JADX and Jarde."""

import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile


HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
JADX_ROOT = Path(os.environ.get("JADX_ROOT", "/Users/lordcasser/workspace/testzone/jadx"))
JADX = Path(os.environ.get("JADX", str(JADX_ROOT / "jadx-cli/build/install/jadx/bin/jadx")))
JARDE_CLI = os.environ.get("JARDE_CLI")
EXPECTED = {"original": "1\n102\n", "jadx": "0\n100\n", "jarde": "1\n102\n"}


def run(args, **kwargs):
    result = subprocess.run(list(map(str, args)), text=True, capture_output=True, **kwargs)
    if result.returncode:
        raise RuntimeError(f"{args[0]} exited {result.returncode}: {result.stderr[-2000:]}")
    return result.stdout


def compile_java(source, classes):
    classes.mkdir()
    run(["javac", "--release", "8", "-g:none", "-Xlint:-options", "-d", classes, source])


with tempfile.TemporaryDirectory(prefix="jarde-cf01-eager-") as temporary:
    work = Path(temporary)
    original = work / "original"
    compile_java(HERE / "EagerBoolean.java", original)
    class_sha256 = hashlib.sha256((original / "EagerBoolean.class").read_bytes()).hexdigest()
    observed = {"original": run(["java", "-Xverify:all", "-cp", original, "EagerBoolean"])}
    bytecode = run(["javap", "-c", "-p", "-classpath", original, "EagerBoolean"])
    if "iand" not in bytecode or "ior" not in bytecode:
        raise RuntimeError("the compiler no longer produces eager integer AND/OR opcodes")
    (HERE / "original-javap.txt").write_text(bytecode)

    jadx_output = work / "jadx"
    run([JADX, "-d", jadx_output, original / "EagerBoolean.class"])
    jadx_files = list(jadx_output.rglob("EagerBoolean.java"))
    if len(jadx_files) != 1:
        raise RuntimeError(f"expected exactly one JADX Java source, got {len(jadx_files)}")
    jadx_source = jadx_files[0].read_text()
    if "if (z && rhs())" not in jadx_source or "if (z || rhs())" not in jadx_source:
        raise RuntimeError("JADX condition transform changed; re-audit the counterexample")
    (HERE / "jadx-EagerBoolean.java.txt").write_text(jadx_source)
    jadx_classes = work / "jadx-classes"
    compile_java(jadx_files[0], jadx_classes)
    observed["jadx"] = run(["java", "-Xverify:all", "-cp", jadx_classes,
                            "defpackage.EagerBoolean"])

    if JARDE_CLI is None:
        environment = os.environ.copy()
        environment["CARGO_TARGET_DIR"] = str(work / "cargo-target")
        environment["CARGO_INCREMENTAL"] = "0"
        environment["CARGO_BUILD_JOBS"] = "2"
        run(["cargo", "build", "-p", "jarde-cli", "--locked"], cwd=ROOT,
            env=environment)
        cli = work / "cargo-target/debug/jarde-cli"
    else:
        cli = Path(JARDE_CLI)
    jarde_source = run([cli, "class-source", "--input", original / "EagerBoolean.class",
                        "--class", "EagerBoolean", "--policy", "single-class",
                        "--format", "text"])
    if "if (arg0 & rhs())" not in jarde_source or "if (arg0 | rhs())" not in jarde_source:
        raise RuntimeError("Jarde eager condition spelling changed; re-audit semantics")
    (HERE / "jarde-EagerBoolean.java.txt").write_text(jarde_source)
    jarde_input = work / "jarde-src/EagerBoolean.java"
    jarde_input.parent.mkdir()
    jarde_input.write_text(jarde_source)
    jarde_classes = work / "jarde-classes"
    compile_java(jarde_input, jarde_classes)
    observed["jarde"] = run(["java", "-Xverify:all", "-cp", jarde_classes,
                             "EagerBoolean"])

    if observed != EXPECTED:
        raise RuntimeError(f"CF-01 results changed: {observed!r}")
    jadx_commit = run(["git", "rev-parse", "HEAD"], cwd=JADX_ROOT).strip()
    (HERE / "results.json").write_text(json.dumps({
        "class_sha256": class_sha256,
        "jadx_commit": jadx_commit,
        "java_release": 8,
        "runtime": observed,
    }, indent=2) + "\n")
    print(json.dumps(observed, indent=2))
