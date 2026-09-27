#!/usr/bin/env python3
"""Replay the two-level-if CF-08 baseline with pinned Java 8/JADX inputs."""

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
import sys
sys.dont_write_bytecode = True

JADX_REV = "2fb1b16386941660fda07e9017285aec40fcb37f"
PINS = {
    "jadx-core/src/test/java/jadx/tests/integration/loops/TestEndlessLoop.java": "a8b077628d7d84d1dd69e51ae2cf0de62210b3f0753461f2c85b0fca829b5df5",
    "jadx-core/src/test/java/jadx/tests/integration/loops/TestEndlessLoop2.java": "1b967881e6f833d1a30e7425a2afb386e149cf73745247df348bb7b6831d298e",
    "jadx-core/src/test/java/jadx/tests/integration/loops/TestMultiEntryLoop.java": "17b9f250a1b32e368b4c8c69b4ec660c9722cf2aee3eb8bd1a6fd63fbc866e74",
    "jadx-core/src/test/java/jadx/tests/integration/loops/TestNotIndexedLoop.java": "dc08e939d5d5ab12c2f4a89e11b266a7e9592ca6289e700241a5eca2f29980b5",
    "jadx-core/src/main/java/jadx/core/dex/visitors/blocks/FixMultiEntryLoops.java": "0c2b67bbb15648632767625db7163bed579efcc66fc3a58b9466760f01eb945a",
    "jadx-core/src/main/java/jadx/core/dex/visitors/regions/maker/LoopRegionMaker.java": "a77bedda10bdb0d36f3979c27c1fc9471da726b36f34ae14d505f6f659c6b13e",
    "jadx-core/src/main/java/jadx/core/dex/visitors/regions/LoopRegionVisitor.java": "921f09e8934fda33452a00359588b293117f160813477e7068fa1ec3b71e6515",
}
EXPECTED = "-1:0\n-1:0\n3:0\n8:1"
SOURCE_SHA256 = "c98dd5aa3d633c96ea9b894ef85cf9665bb7365b7dc74d29418d1255244f44ce"
RUNNER_SHA256 = "303e2d371c2d724ab33397bc946bd57e0fa954bf60f35ed7341ea3c6af04b368"
CLASS_SHA256 = "b5a7541fc6dc6a6eaff229f3559968cd7e20a4f48c063e4fd019186f92b4a240"


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def invoke(command, log, timeout=180):
    result = subprocess.run([str(part) for part in command], capture_output=True,
                            text=True, timeout=timeout)
    log.parent.mkdir(parents=True, exist_ok=True)
    stdout = "\n".join(line.rstrip() for line in result.stdout.splitlines())
    stderr = "\n".join(line.rstrip() for line in result.stderr.splitlines())
    lines = [f"exit={result.returncode}", "stdout:", *stdout.splitlines(), "stderr:",
             *stderr.splitlines()]
    log.write_text("\n".join(lines) + "\n")
    return result


def compile_run(label, source, runner, out, temp):
    classes = temp / f"{label}-classes"
    classes.mkdir()
    compiled = invoke(["javac", "--release", "8", "-g:none", "-d", classes,
                       source, runner], out / label / "javac.log")
    answer = {"source_sha256": digest(source), "javac_exit": compiled.returncode,
              "javac_stderr": compiled.stderr.strip()}
    if compiled.returncode:
        return answer, classes
    executed = invoke(["java", "-Xverify:all", "-cp", classes, "cf08twolvl.Runner"],
                      out / label / "runtime.log", timeout=5)
    answer.update(runtime_exit=executed.returncode, stdout=executed.stdout.strip(),
                  runtime_stderr=executed.stderr.strip())
    return answer, classes


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("jarde", "jadx", "jadx-checkout", "out"):
        parser.add_argument("--" + name, required=True, type=Path)
    parser.add_argument("--require-jarde", action="store_true")
    args = parser.parse_args()
    out = args.out.resolve()
    if out.exists() and any(out.iterdir()):
        parser.error("--out must be absent or empty")
    out.mkdir(parents=True, exist_ok=True)

    rev = invoke(["git", "-C", args.jadx_checkout, "rev-parse", "HEAD"],
                 out / "jadx-revision.log")
    if rev.returncode or rev.stdout.strip() != JADX_REV:
        raise RuntimeError("JADX checkout differs from pinned revision")
    for path, expected in PINS.items():
        if digest(args.jadx_checkout / path) != expected:
            raise RuntimeError(f"pinned JADX source changed: {path}")

    source = HERE / "input/cf08twolvl/TwoLevelIf.java"
    runner = HERE / "input/cf08twolvl/Runner.java"
    if digest(source) != SOURCE_SHA256 or digest(runner) != RUNNER_SHA256:
        raise RuntimeError("pinned Java input changed")
    with tempfile.TemporaryDirectory(prefix="jarde-cf08-two-level-") as temp_name:
        temp = Path(temp_name)
        original, classes = compile_run("original", source, runner, out, temp)
        if original.get("runtime_exit") or original.get("stdout") != EXPECTED:
            raise RuntimeError("original Java 8 output differs")
        class_file = classes / "cf08twolvl/TwoLevelIf.class"
        if digest(class_file) != CLASS_SHA256:
            raise RuntimeError("frozen Java 8 class changed")
        frozen_class = out / "classes/cf08twolvl/TwoLevelIf.class"
        frozen_class.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(class_file, frozen_class)
        if invoke(["javap", "-classpath", classes, "-c", "-p", "cf08twolvl.TwoLevelIf"],
                  out / "javap.log").returncode:
            raise RuntimeError("javap failed")

        regions_run = invoke([args.jarde, "recover", "--input", class_file,
                              "--class-name", "cf08twolvl.TwoLevelIf", "--method-name", "pick",
                              "--descriptor", "([I)I", "--policy", "single-class",
                              "--release", "8", "--evidence", "region_details", "--format", "json"],
                             out / "region-report.log")
        if regions_run.returncode:
            raise RuntimeError("Jarde region evidence failed")
        recovery = json.loads(regions_run.stdout)["recovered"]["recovery"]
        region_summary = {"representation": recovery["representation"], "quality": recovery["quality"],
                          "regions": recovery["regions"], "diagnostics": recovery["diagnostics"]}
        (out / "regions.json").write_text(json.dumps(region_summary, indent=2) + "\n")

        jar = temp / "input.jar"
        if invoke(["jar", "cf", jar, "-C", classes, "."], out / "jar.log").returncode:
            raise RuntimeError("jar failed")
        jadx_root = temp / "jadx"
        if invoke([args.jadx, "-d", jadx_root, jar], out / "jadx.log").returncode:
            raise RuntimeError("JADX failed")
        jadx_source = out / "source/jadx/cf08twolvl/TwoLevelIf.java"
        jadx_source.parent.mkdir(parents=True)
        shutil.copy2(jadx_root / "sources/cf08twolvl/TwoLevelIf.java", jadx_source)
        jarde_run = invoke([args.jarde, "class-source", "--input", jar,
                            "--class", "cf08twolvl.TwoLevelIf", "--policy", "plain-jar",
                            "--release", "8", "--format", "text"], out / "jarde.log")
        if jarde_run.returncode:
            raise RuntimeError("Jarde class-source command failed")
        jarde_source = out / "source/jarde/cf08twolvl/TwoLevelIf.java"
        jarde_source.parent.mkdir(parents=True)
        jarde_source.write_text(jarde_run.stdout)
        jadx, _ = compile_run("jadx", jadx_source, runner, out, temp)
        jarde, _ = compile_run("jarde", jarde_source, runner, out, temp)
        if jadx.get("javac_exit") or jadx.get("runtime_exit") or jadx.get("stdout") != EXPECTED:
            raise RuntimeError("fixed JADX Java 8 replay differs")
        summary = {"jadx_revision": JADX_REV, "jadx_pins": PINS,
                   "input_sha256": digest(source), "runner_sha256": digest(runner),
                   "original_class_sha256": digest(class_file), "jarde_cli_sha256": digest(args.jarde),
                   "original": original, "jadx": jadx, "jarde": jarde}
        (out / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
        print(json.dumps(summary, indent=2))

        uncovered = [region for region in recovery["regions"]
                     if region["code"] == "jre_region_uncovered_blocks"]
        refusal = "local 1 crosses a quoted fallback region"
        if args.require_jarde:
            if jarde.get("javac_exit") or jarde.get("runtime_exit") or jarde.get("stdout") != EXPECTED:
                raise RuntimeError("Jarde complete Java 8 replay differs")
            if "@bytecode" in jarde_source.read_text():
                raise RuntimeError("Jarde still quotes pick bytecode")
        elif (jarde.get("javac_exit") == 0 or "@bytecode" not in jarde_source.read_text()
              or refusal not in jarde_source.read_text()
              or len(uncovered) != 1
              or set(uncovered[0]["blocks"]) != {23, 28, 37, 46, 49}
              or not any(d["code"] == "jre_region_arms_do_not_meet"
                         and "block 0 do not meet at one join" in d["message"]
                         for d in recovery["diagnostics"])):
            raise RuntimeError("expected Jarde region/refusal baseline changed")


if __name__ == "__main__":
    main()
