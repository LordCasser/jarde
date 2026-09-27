#!/usr/bin/env python3
"""DT-24 top-level mixed annotation-value control against fixed JADX."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
import zipfile

HERE = Path(__file__).resolve().parent
JADX_ROOT = Path(os.environ.get("JADX_ROOT", "/Users/lordcasser/workspace/testzone/jadx"))
JADX = Path(os.environ.get("JADX", str(JADX_ROOT / "jadx-cli/build/install/jadx/bin/jadx")))
CLI = Path(os.environ["JARDE_CLI"])
HEAD = "2fb1b16386941660fda07e9017285aec40fcb37f"
NAMES = ("Simple", "Mode", "Mix", "Tagged")
SOURCES = [HERE / f"{name}.java" for name in (*NAMES, "Runner")]
EXPECTED = "b:7:9.87:[0.0, 1.1]:java.lang.Exception:TWO:false:[3, 5]\n"


def call(args, cwd=None):
    return subprocess.run([str(arg) for arg in args], cwd=cwd, text=True, capture_output=True)


def check(ok, message):
    if not ok:
        raise RuntimeError(message)


def compile_run(sources, output):
    output.mkdir(parents=True)
    compiled = call(["javac", "-J-Duser.language=en", "-J-Duser.country=US", "--release", "8",
                     "-Xlint:-options", "-d", output, *sources])
    check(compiled.returncode == 0, compiled.stderr)
    executed = call(["java", "-Xverify:all", "-cp", output, "dt24.Runner"])
    check(executed.returncode == 0 and executed.stdout == EXPECTED,
          executed.stderr or executed.stdout)
    return {"compile_exit": compiled.returncode, "run_exit": executed.returncode,
            "run_stdout": executed.stdout}


with tempfile.TemporaryDirectory(prefix="jarde-dt24-values-") as raw:
    work = Path(raw)
    original_classes = work / "original-classes"
    original = compile_run(SOURCES, original_classes)
    classes = {}
    with zipfile.ZipFile(work / "input.jar", "w", zipfile.ZIP_DEFLATED) as jar:
        for path in sorted((original_classes / "dt24").glob("*.class")):
            classes[path.stem] = hashlib.sha256(path.read_bytes()).hexdigest()
            if path.stem != "Runner":
                jar.write(path, "dt24/" + path.name)
    actual_head = call(["git", "rev-parse", "HEAD"], JADX_ROOT)
    check(actual_head.returncode == 0 and actual_head.stdout.strip() == HEAD,
          "JADX checkout is not the fixed inventory revision")
    jadx_dir = work / "jadx"
    jadx_result = call([JADX, "-d", jadx_dir, work / "input.jar"])
    check(jadx_result.returncode == 0, jadx_result.stderr)
    jadx = compile_run(sorted(jadx_dir.rglob("*.java")) + [HERE / "Runner.java"],
                       work / "jadx-classes")
    for name in NAMES:
        (HERE / f"jadx-{name}.java.txt").write_text(
            (jadx_dir / "sources/dt24" / f"{name}.java").read_text())

    jarde_dir = work / "jarde/dt24"
    jarde_dir.mkdir(parents=True)
    for name in NAMES:
        result = call([CLI, "class-source", "--input", work / "input.jar", "--class",
                       "dt24/" + name, "--policy", "plain-jar", "--release", "8",
                       "--format", "text"])
        check(result.returncode == 0, result.stderr)
        (jarde_dir / f"{name}.java").write_text(result.stdout)
        (HERE / f"jarde-{name}.java.txt").write_text(result.stdout)
    jarde = compile_run(sorted(jarde_dir.glob("*.java")) + [HERE / "Runner.java"],
                        work / "jarde-classes")
    tagged = (jarde_dir / "Tagged.java").read_text()
    check("@dt24.Mix(" in tagged
          and "value = 0x1.3bd70ap3f" in tagged
          and "doubles = {0.0d, 0x1.199999999999ap0d}" in tagged
          and "cls = java.lang.Exception.class" in tagged
          and "mode = dt24.Mode.TWO" in tagged
          and "nested = @dt24.Simple(value = false)" in tagged
          and "ints = {3, 5}" in tagged,
          "one or more annotation value kinds disappeared from the Jarde declaration")
    result = {
        "jadx_commit": HEAD,
        "javac": call(["javac", "-version"]).stdout.strip(),
        "source_sha256": {path.name: hashlib.sha256(path.read_bytes()).hexdigest()
                          for path in SOURCES},
        "class_sha256": classes,
        "original": original,
        "jadx": jadx,
        "jarde": jarde,
        "assertions": {"complete_java8_verified_equal": True,
                       "all_selected_annotation_value_kinds_present": True},
    }
    (HERE / "results.json").write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps(result["assertions"], ensure_ascii=False))
