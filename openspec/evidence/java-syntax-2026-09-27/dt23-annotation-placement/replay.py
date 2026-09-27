#!/usr/bin/env python3
"""DT-23 top-level declaration/field/method/parameter annotation control."""
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
SOURCES = [HERE / name for name in ("A.java", "Types.java", "Runner.java")]
EXPECTED = "class=true\nfield=true\nmethod=true\nparam0=true\nparam1=5\n"


def call(args, cwd=None):
    return subprocess.run([str(arg) for arg in args], cwd=cwd, text=True, capture_output=True)


def check(ok, message):
    if not ok:
        raise RuntimeError(message)


def compile_and_run(sources, output):
    output.mkdir(parents=True)
    compiled = call(["javac", "-J-Duser.language=en", "-J-Duser.country=US", "--release", "8",
                     "-g:none", "-Xlint:-options", "-d", output, *sources])
    check(compiled.returncode == 0, compiled.stderr)
    run = call(["java", "-Xverify:all", "-cp", output, "dt23.Runner"])
    check(run.returncode == 0 and run.stdout == EXPECTED, run.stderr or run.stdout)
    return {"compile_exit": compiled.returncode, "run_exit": run.returncode,
            "run_stdout": run.stdout}


with tempfile.TemporaryDirectory(prefix="jarde-dt23-annotations-") as raw:
    work = Path(raw)
    original_classes = work / "original-classes"
    original = compile_and_run(SOURCES, original_classes)
    classes = {}
    with zipfile.ZipFile(work / "input.jar", "w", zipfile.ZIP_DEFLATED) as jar:
        for path in sorted((original_classes / "dt23").glob("*.class")):
            classes[path.stem] = hashlib.sha256(path.read_bytes()).hexdigest()
            if path.stem != "Runner":
                jar.write(path, "dt23/" + path.name)
    actual_head = call(["git", "rev-parse", "HEAD"], JADX_ROOT)
    check(actual_head.returncode == 0 and actual_head.stdout.strip() == HEAD,
          "JADX checkout is not the fixed inventory revision")
    jadx_dir = work / "jadx"
    jadx_run = call([JADX, "-d", jadx_dir, work / "input.jar"])
    check(jadx_run.returncode == 0, jadx_run.stderr)
    jadx = compile_and_run(sorted(jadx_dir.rglob("*.java")) + [HERE / "Runner.java"],
                           work / "jadx-classes")
    for name in ("A", "Types"):
        (HERE / f"jadx-{name}.java.txt").write_text(
            (jadx_dir / "sources/dt23" / f"{name}.java").read_text())

    jarde_dir = work / "jarde/dt23"
    jarde_dir.mkdir(parents=True)
    for name in ("A", "Types"):
        output = call([CLI, "class-source", "--input", work / "input.jar", "--class",
                       "dt23/" + name, "--policy", "plain-jar", "--release", "8",
                       "--format", "text"])
        check(output.returncode == 0, output.stderr)
        (jarde_dir / f"{name}.java").write_text(output.stdout)
        (HERE / f"jarde-{name}.java.txt").write_text(output.stdout)
    jarde = compile_and_run(sorted(jarde_dir.glob("*.java")) + [HERE / "Runner.java"],
                            work / "jarde-classes")
    recovered = (jarde_dir / "Types.java").read_text()
    check(recovered.count("@dt23.A(c = dt23.Types.class)") == 4,
          "class, field, method, first parameter annotation placement changed")
    check("@dt23.A(c = dt23.Types.class, i = 5)" in recovered,
          "second parameter's explicit value changed")
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
                       "four_annotation_sites_and_two_parameter_positions": True},
    }
    (HERE / "results.json").write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps(result["assertions"], ensure_ascii=False))
