#!/usr/bin/env python3
"""Replay fixed JADX CF-08 endless/multi-entry loop evidence against Java 8 and Jarde."""

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile


HERE = Path(__file__).resolve().parent
INPUT = HERE / "input/cf08"
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
EXPECTED = "null\nnull\nf\nh"


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def run(command, log, *, compact=False):
    result = subprocess.run([str(part) for part in command], capture_output=True,
                            text=True, timeout=180)
    log.parent.mkdir(parents=True, exist_ok=True)
    if compact:
        log.write_text(f"exit={result.returncode}\n"
                       f"stdout_sha256={hashlib.sha256(result.stdout.encode()).hexdigest()}\n"
                       f"stderr:\n{result.stderr}")
    else:
        content = f"exit={result.returncode}\nstdout:\n{result.stdout}\nstderr:\n{result.stderr}"
        log.write_text("\n".join(line.rstrip() for line in content.splitlines()) + "\n")
    return result


def compile_run(label, source, runner, out, temp):
    classes = temp / f"{label}-classes"
    classes.mkdir()
    compiled = run(["javac", "--release", "8", "-g:none", "-d", classes,
                    source, runner], out / label / "javac.log")
    answer = {"source_sha256": digest(source), "javac_exit": compiled.returncode,
              "javac_stderr": compiled.stderr.strip()}
    if compiled.returncode:
        return answer, classes
    executed = run(["java", "-Xverify:all", "-cp", classes, "cf08.Runner"],
                   out / label / "runtime.log")
    answer.update(runtime_exit=executed.returncode,
                  stdout=executed.stdout.strip(),
                  runtime_stderr=executed.stderr.strip())
    return answer, classes


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("jarde", "jadx", "jadx-checkout", "out"):
        parser.add_argument("--" + name, required=True, type=Path)
    args = parser.parse_args()
    out = args.out.resolve()
    if out.exists() and any(out.iterdir()):
        parser.error("--out must be absent or empty")
    out.mkdir(parents=True, exist_ok=True)
    revision = run(["git", "-C", args.jadx_checkout, "rev-parse", "HEAD"],
                   out / "jadx-revision.log")
    if revision.returncode or revision.stdout.strip() != JADX_REV:
        raise RuntimeError("JADX checkout changed")
    for path, expected in PINS.items():
        if digest(args.jadx_checkout / path) != expected:
            raise RuntimeError(f"pinned JADX source changed: {path}")

    source = INPUT / "NotIndexedLoop.java"
    runner = INPUT / "Runner.java"
    with tempfile.TemporaryDirectory(prefix="jarde-cf08-") as temp_name:
        temp = Path(temp_name)
        original, classes = compile_run("original", source, runner, out, temp)
        if original.get("runtime_exit") or original.get("stdout") != EXPECTED:
            raise RuntimeError("original Java 8 fixture changed")
        class_file = classes / "cf08/NotIndexedLoop.class"
        class_digest = digest(class_file)
        if run(["javap", "-classpath", classes, "-c", "-p", "cf08.NotIndexedLoop"],
               out / "javap.log").returncode:
            raise RuntimeError("javap failed")
        jar = temp / "input.jar"
        if run(["jar", "cf", jar, "-C", classes, "."], out / "jar.log").returncode:
            raise RuntimeError("jar failed")
        jadx_root = temp / "jadx"
        if run([args.jadx, "-d", jadx_root, jar], out / "jadx.log").returncode:
            raise RuntimeError("JADX failed")
        jadx_source = out / "source/jadx/cf08/NotIndexedLoop.java"
        jadx_source.parent.mkdir(parents=True)
        shutil.copy2(jadx_root / "sources/cf08/NotIndexedLoop.java", jadx_source)
        recovered = run([args.jarde, "class-source", "--input", jar,
                         "--class", "cf08.NotIndexedLoop", "--policy", "plain-jar",
                         "--release", "8", "--format", "text"],
                        out / "jarde.log", compact=True)
        if recovered.returncode:
            raise RuntimeError("Jarde CLI failed")
        jarde_source = out / "source/jarde/cf08/NotIndexedLoop.java"
        jarde_source.parent.mkdir(parents=True)
        jarde_source.write_text(recovered.stdout)
        jadx, _ = compile_run("jadx", jadx_source, runner, out, temp)
        jarde, _ = compile_run("jarde", jarde_source, runner, out, temp)
        summary = {
            "jadx_revision": JADX_REV,
            "jadx_pins": PINS,
            "input_sha256": digest(source),
            "runner_sha256": digest(runner),
            "original_class_sha256": class_digest,
            "jarde_cli_sha256": digest(args.jarde),
            "original": original,
            "jadx": jadx,
            "jarde": jarde,
        }
        (out / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
        print(json.dumps(summary, indent=2))
        if jadx.get("javac_exit") or jadx.get("runtime_exit") or jadx.get("stdout") != EXPECTED:
            raise RuntimeError("complete JADX Java 8 replay differs")


if __name__ == "__main__":
    main()
