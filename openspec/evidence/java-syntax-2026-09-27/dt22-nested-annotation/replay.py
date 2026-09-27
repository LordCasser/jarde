#!/usr/bin/env python3
"""Fixed Java 8 comparison for DT-22's annotation owner and default value."""
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import zipfile

HERE = Path(__file__).resolve().parent
JADX_ROOT = Path(os.environ.get("JADX_ROOT", "/Users/lordcasser/workspace/testzone/jadx"))
JADX = Path(os.environ.get("JADX", str(JADX_ROOT / "jadx-cli/build/install/jadx/bin/jadx")))
CLI = Path(os.environ["JARDE_CLI"])
EXPECTED_JADX = "2fb1b16386941660fda07e9017285aec40fcb37f"
INPUTS = [HERE / name for name in ("Holder.java", "TopDefault.java", "Dollar$A.java", "Runner.java")]


def run(args):
    return subprocess.run([str(arg) for arg in args], text=True, capture_output=True)


def require(ok, message):
    if not ok:
        raise RuntimeError(message)


def java_compile(sources, out):
    out.mkdir(parents=True)
    return run(["javac", "-J-Duser.language=en", "-J-Duser.country=US", "--release", "8",
                "-Xlint:-options", "-d", out, *sources])


def compile_run(sources, out):
    compile_result = java_compile(sources, out)
    result = {"compile_exit": compile_result.returncode}
    if compile_result.returncode:
        result["compile_diagnostic"] = (compile_result.stderr
                                        .replace(str(out.parent), "<TMP>")
                                        .replace(str(HERE), "<FIXTURE>"))
        return result
    execution = run(["java", "-Xverify:all", "-cp", out, "dt22.Runner"])
    result.update(run_exit=execution.returncode, run_stdout=execution.stdout)
    require(execution.returncode == 0, execution.stderr)
    return result


with tempfile.TemporaryDirectory(prefix="jarde-dt22-annotation-") as temporary:
    work = Path(temporary)
    original_classes = work / "original-classes"
    original = compile_run(INPUTS, original_classes)
    require(original["compile_exit"] == 0, str(original))
    javap = []
    for name in ("Holder", "Holder$A", "TopDefault", "Dollar$A"):
        dump = run(["javap", "-v", "-p", "-classpath", original_classes, "dt22." + name])
        require(dump.returncode == 0, dump.stderr)
        stable = dump.stdout.replace(str(original_classes), "<TMP>")
        stable = re.sub(r"(?m)^  Last modified .*; size (\d+) bytes$",
                        r"  Last modified <NORMALIZED>; size \1 bytes", stable)
        javap.append(stable)
    (HERE / "original-javap.txt").write_text("\n".join(javap))

    class_hashes = {}
    with zipfile.ZipFile(work / "input.jar", "w", zipfile.ZIP_DEFLATED) as jar:
        for path in sorted((original_classes / "dt22").glob("*.class")):
            class_hashes[path.stem] = hashlib.sha256(path.read_bytes()).hexdigest()
            if path.stem != "Runner":
                jar.write(path, "dt22/" + path.name)

    jadx_head = subprocess.run(["git", "rev-parse", "HEAD"], cwd=JADX_ROOT,
                               text=True, capture_output=True)
    require(jadx_head.returncode == 0 and jadx_head.stdout.strip() == EXPECTED_JADX,
            "JADX checkout is not the fixed inventory revision")
    jadx_output = work / "jadx"
    jadx_result = run([JADX, "-d", jadx_output, work / "input.jar"])
    require(jadx_result.returncode == 0, jadx_result.stderr)
    jadx = compile_run(sorted(jadx_output.rglob("*.java")) + [HERE / "Runner.java"],
                       work / "jadx-classes")
    require(jadx == original, "fixed JADX complete source or runtime differs from original")
    for name in ("Holder", "TopDefault", "Dollar$A"):
        source = jadx_output / "sources/dt22" / f"{name}.java"
        require(source.is_file(), f"fixed JADX source is missing: {name}")
        (HERE / f"jadx-{name.replace('$', '_')}.java.txt").write_text(source.read_text())

    jarde_sources = work / "jarde-sources" / "dt22"
    jarde_sources.mkdir(parents=True)
    child_text = ""
    jarde_headers = {}
    for name in ("Holder", "Holder$A", "TopDefault", "Dollar$A"):
        result = run([CLI, "class-source", "--input", work / "input.jar", "--class", "dt22/" + name,
                      "--policy", "plain-jar", "--release", "8", "--format", "text"])
        require(result.returncode == 0, f"Jarde could not read {name}: {result.stderr}")
        (HERE / f"jarde-{name.replace('$', '_')}.java.txt").write_text(result.stdout)
        jarde_headers[name] = next(line for line in result.stdout.splitlines()
                                   if line.startswith("public "))
        if name == "Holder$A":
            child_text = result.stdout
        else:
            (jarde_sources / f"{name}.java").write_text(result.stdout)

    jarde_only = java_compile(sorted(jarde_sources.glob("*.java")), work / "jarde-only-classes")
    require(jarde_only.returncode == 0, jarde_only.stderr)
    jarde = compile_run(sorted(jarde_sources.glob("*.java")) + [HERE / "Runner.java"],
                        work / "jarde-runner-classes")
    require(jarde["compile_exit"] != 0 and "symbol:   class A" in jarde["compile_diagnostic"],
            "baseline Jarde must fail only at the original Holder.A source API")
    require("public @interface Holder$A" in child_text,
            "independent child declaration must keep its physical name")
    require("default 0x1.19999ap0f" in child_text
            and "default 0x1.19999ap0f" in (jarde_sources / "TopDefault.java").read_text(),
            "floating AnnotationDefault must be preserved in both physical declarations")
    require("@interface A" in (jadx_output / "sources/dt22/Holder.java").read_text(),
            "fixed JADX must emit A inside Holder")
    report = {
        "jadx_commit": EXPECTED_JADX,
        "javac": run(["javac", "-version"]).stdout.strip(),
        "source_sha256": {path.name: hashlib.sha256(path.read_bytes()).hexdigest() for path in INPUTS},
        "class_sha256": class_hashes,
        "original": original,
        "jadx": jadx,
        "jarde_sources_only_compile_exit": jarde_only.returncode,
        "jarde_original_api": jarde,
        "jarde_headers": jarde_headers,
        "assertions": {
            "original_jadx_java8_verified_equal": True,
            "jarde_top_level_defaults_preserved": True,
            "jarde_physical_nested_default_preserved": True,
            "jarde_nested_owner_source_api_missing": True,
        },
    }
    (HERE / "results.json").write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps(report["assertions"], ensure_ascii=False))
