#!/usr/bin/env python3
"""Freeze DT-26 captured-lambda source and complete Java 8 recompilation outcomes."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
from zipfile import ZIP_DEFLATED, ZipFile, ZipInfo

HERE = Path(__file__).resolve().parent
OUT = HERE / "outputs"
JADX_ROOT = Path(os.environ.get("JADX_ROOT", "/Users/lordcasser/workspace/testzone/jadx"))
JADX = Path(os.environ.get("JADX", str(JADX_ROOT / "jadx-cli/build/install/jadx/bin/jadx")))
CLI = Path(os.environ["JARDE_CLI"])
JADX_HEAD = "2fb1b16386941660fda07e9017285aec40fcb37f"
EXPECTED = "7:-1\n"


def run(*args, cwd=None):
    return subprocess.run([str(arg) for arg in args], cwd=cwd, text=True, capture_output=True)


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def compile_source(label, sources, directory, expect_success):
    classes = directory / f"{label}-classes"
    classes.mkdir(parents=True)
    compiled = run("javac", "-J-Duser.language=en", "-J-Duser.country=US",
                   "--release", "8", "-g:none", "-Xlint:-options", "-d", classes, *sources)
    require((compiled.returncode == 0) == expect_success,
            f"{label} compile result changed: {compiled.stderr}")
    result = {"compile_exit": compiled.returncode,
              "compile_stderr": compiled.stderr.replace(str(directory), "<WORK>")}
    if expect_success:
        executed = run("java", "-Xverify:all", "-cp", classes, "dt26.Runner")
        require(executed.returncode == 0 and executed.stdout == EXPECTED,
                f"{label} verified output changed: {executed.stderr or executed.stdout}")
        result.update(run_exit=executed.returncode, run_stdout=executed.stdout)
    return result, classes


OUT.mkdir(exist_ok=True)
head = run("git", "rev-parse", "HEAD", cwd=JADX_ROOT)
require(head.returncode == 0 and head.stdout.strip() == JADX_HEAD,
        "JADX checkout is not the pinned inventory revision")
sources = [HERE / "CaptureCases.java", HERE / "Runner.java"]
with tempfile.TemporaryDirectory(prefix="jarde-dt26-capture-") as raw:
    work = Path(raw)
    original, original_classes = compile_source("original", sources, work, True)
    class_file = original_classes / "dt26/CaptureCases.class"
    archive = work / "input.jar"
    with ZipFile(archive, "w", ZIP_DEFLATED) as jar:
        entry = ZipInfo("dt26/CaptureCases.class", date_time=(2000, 1, 1, 0, 0, 0))
        entry.compress_type = ZIP_DEFLATED
        jar.writestr(entry, class_file.read_bytes())
    javap = run("javap", "-c", "-p", "-classpath", original_classes,
                "dt26.CaptureCases")
    require(javap.returncode == 0, javap.stderr)
    (OUT / "javap-CaptureCases.txt").write_text(javap.stdout)

    jadx_dir = work / "jadx"
    decompiled = run(JADX, "-d", jadx_dir, archive)
    require(decompiled.returncode == 0, decompiled.stderr)
    jadx_file = jadx_dir / "sources/dt26/CaptureCases.java"
    require(jadx_file.is_file(), "JADX did not emit CaptureCases")
    jadx_text = jadx_file.read_text()
    (OUT / "jadx-CaptureCases.java.txt").write_text(jadx_text)
    jadx, _ = compile_source("jadx", [jadx_file, HERE / "Runner.java"], work, True)

    generated = run(CLI, "class-source", "--input", archive,
                    "--class", "dt26/CaptureCases", "--policy", "plain-jar",
                    "--release", "8", "--format", "text")
    require(generated.returncode == 0, generated.stderr)
    jarde_text = generated.stdout
    (OUT / "jarde-CaptureCases.java.txt").write_text(jarde_text)
    jarde_file = work / "jarde-src/dt26/CaptureCases.java"
    jarde_file.parent.mkdir(parents=True)
    jarde_file.write_text(jarde_text)
    jarde, _ = compile_source("jarde", [jarde_file, HERE / "Runner.java"], work, True)
    require("lambda$" not in jarde_text,
            "Jarde source still declares or calls a compiler-synthetic helper")

    result = {
        "jadx_commit": JADX_HEAD,
        "source_sha256": {path.name: hashlib.sha256(path.read_bytes()).hexdigest()
                          for path in sources},
        "class_sha256": hashlib.sha256(class_file.read_bytes()).hexdigest(),
        "decompiled_sha256": {
            "jadx": hashlib.sha256(jadx_text.encode()).hexdigest(),
            "jarde": hashlib.sha256(jarde_text.encode()).hexdigest(),
        },
        "original": original, "jadx": jadx, "jarde": jarde,
        "assertions": {
            "original_and_jadx_complete_java8_sources_verify_equally": True,
            "jarde_captured_lambda_helpers_are_inlined_and_java8_recompile_verifies": True,
        },
    }
    (OUT / "results.json").write_text(json.dumps(result, ensure_ascii=False,
                                                   indent=2, sort_keys=True) + "\n")
    print(json.dumps(result["assertions"], ensure_ascii=False, sort_keys=True))
