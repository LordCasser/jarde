#!/usr/bin/env python3
"""Replay the bounded CF-01 nested/negated short-circuit CFG comparison."""
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
SOURCE = HERE / "ShortCircuitNegation.java"
RUNNER = HERE / "Runner.java"
CLASS_NAME = "probe.ShortCircuitNegation"

def call(args, **kwargs):
    return subprocess.run(list(map(str, args)), text=True, capture_output=True, **kwargs)

def checked(args, **kwargs):
    result = call(args, **kwargs)
    if result.returncode:
        raise RuntimeError(f"{args[0]} exited {result.returncode}: {result.stderr[-3000:]}")
    return result.stdout

def compile_pair(source, runner, classes):
    classes.mkdir()
    return call(["javac", "--release", "8", "-g:none", "-Xlint:-options", "-d", classes, source, runner])

def compile_and_run(source, runner, label, work):
    source_path = work / f"{label}-src" / "probe" / "ShortCircuitNegation.java"
    runner_path = source_path.parent / "Runner.java"
    source_path.parent.mkdir(parents=True)
    source_path.write_text(source)
    runner_path.write_text(runner)
    classes = work / f"{label}-classes"
    compile_result = compile_pair(source_path, runner_path, classes)
    record = {"compile_exit": compile_result.returncode}
    if compile_result.returncode:
        record["compile_stderr"] = compile_result.stderr
        return record
    execution = call(["java", "-Xverify:all", "-cp", classes, "probe.Runner"])
    record.update({"run_exit": execution.returncode,
                   "run_stdout_sha256": hashlib.sha256(execution.stdout.encode()).hexdigest(),
                   "run_stdout_bytes": len(execution.stdout.encode())})
    if execution.returncode:
        record["run_stderr"] = execution.stderr
    return record, execution.stdout

with tempfile.TemporaryDirectory(prefix="jarde-cf01-short-circuit-") as tmp:
    work = Path(tmp)
    original_classes = work / "original-classes"
    original_compile = compile_pair(SOURCE, RUNNER, original_classes)
    if original_compile.returncode:
        raise RuntimeError(original_compile.stderr)
    class_file = original_classes / "probe/ShortCircuitNegation.class"
    bytecode = checked(["javap", "-c", "-p", "-classpath", original_classes, CLASS_NAME])
    if "ifeq" not in bytecode or "ifne" not in bytecode:
        raise RuntimeError("javac did not emit the expected short-circuit conditional branches")
    (HERE / "original-javap.txt").write_text(bytecode)
    original_run = call(["java", "-Xverify:all", "-cp", original_classes, "probe.Runner"])
    if original_run.returncode:
        raise RuntimeError(original_run.stderr)

    jadx_commit = checked(["git", "rev-parse", "HEAD"], cwd=JADX_ROOT).strip()
    expected_jadx_commit = "2fb1b16386941660fda07e9017285aec40fcb37f"
    if jadx_commit != expected_jadx_commit:
        raise RuntimeError(f"expected fixed JADX commit {expected_jadx_commit}, got {jadx_commit}")
    jadx_dir = work / "jadx"
    checked([JADX, "-d", jadx_dir, class_file])
    jadx_files = list(jadx_dir.rglob("ShortCircuitNegation.java"))
    if len(jadx_files) != 1:
        raise RuntimeError(f"expected one JADX source, got {jadx_files}")
    jadx_source = jadx_files[0].read_text()
    jadx_result, jadx_output = compile_and_run(jadx_source, RUNNER.read_text(), "jadx", work)
    (HERE / "jadx-ShortCircuitNegation.java.txt").write_text(jadx_source)

    if JARDE_CLI is None:
        env = os.environ.copy()
        env["CARGO_TARGET_DIR"] = str(work / "cargo-target")
        env["CARGO_INCREMENTAL"] = "0"
        env["CARGO_BUILD_JOBS"] = "2"
        build = call(["cargo", "build", "-p", "jarde-cli", "--locked"], cwd=ROOT, env=env)
        if build.returncode:
            raise RuntimeError(build.stderr[-5000:])
        cli = work / "cargo-target/debug/jarde-cli"
    else:
        cli = Path(JARDE_CLI)
    jarde = checked([cli, "class-source", "--input", class_file, "--class", "probe/ShortCircuitNegation",
                     "--policy", "single-class", "--release", "8", "--format", "text"])
    if "@bytecode" in jarde:
        raise RuntimeError("Jarde full class source contains an unresolved bytecode fallback")
    for fragment in ["&&", "||", "!probe('a'", "negatedConditions"]:
        if fragment not in jarde:
            raise RuntimeError(f"Jarde output lost expected logical fragment {fragment!r}")
    (HERE / "jarde-ShortCircuitNegation.java.txt").write_text(jarde)
    jarde_result, jarde_output = compile_and_run(jarde, RUNNER.read_text(), "jarde", work)
    original_output = original_run.stdout
    (HERE / "original-run.txt").write_text(original_output)
    if not (original_output == jadx_output == jarde_output):
        raise RuntimeError("original/JADX/Jarde runtime outputs differ")
    for label, result in (("JADX", jadx_result), ("Jarde", jarde_result)):
        if result["compile_exit"] != 0 or result["run_exit"] != 0:
            raise RuntimeError(f"{label} full-source Java 8 replay failed: {result}")

    record = {
        "class_sha256": hashlib.sha256(class_file.read_bytes()).hexdigest(),
        "javac": checked(["javac", "-version"]).strip(),
        "java_version": call(["java", "-version"]).stderr.strip(),
        "jadx_commit": jadx_commit,
        "source_sha256": hashlib.sha256(SOURCE.read_bytes()).hexdigest(),
        "runner_sha256": hashlib.sha256(RUNNER.read_bytes()).hexdigest(),
        "javap_sha256": hashlib.sha256(bytecode.encode()).hexdigest(),
        "original": {"compile_exit": 0, "run_exit": original_run.returncode,
                     "run_stdout_sha256": hashlib.sha256(original_run.stdout.encode()).hexdigest(),
                     "run_stdout_bytes": len(original_run.stdout.encode())},
        "jadx": jadx_result,
        "jarde": jarde_result,
        "all_stdout_equal": original_output == jadx_output == jarde_output,
    }
    (HERE / "results.json").write_text(json.dumps(record, indent=2) + "\n")
    print(json.dumps(record, indent=2))
