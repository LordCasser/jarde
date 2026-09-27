#!/usr/bin/env python3
"""Replay the legal Java 8 construction-chain slice of fixed JADX EM-05 tests."""

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
INPUT = HERE / "input" / "em05"
JADX_REV = "2fb1b16386941660fda07e9017285aec40fcb37f"
PINS = {
    "test/java/jadx/tests/integration/others/TestConstructorBranched.java": "fbbaca8b476cc19da002350aad1e7b123e00ecdde47ccd96aa4dfcd0eaf7ca11",
    "test/java/jadx/tests/integration/others/TestDefConstructorNotRemoved.java": "10866f78029a09dfe7e13a1b3ee9c9dc869a43e80bd233dfc0bb15c3013ba62d",
    "test/java/jadx/tests/integration/others/TestInsnsBeforeSuper2.java": "dfe805b6728f11d160885b04ab09adcc17a0e70229fd1d45b4acbe01849b2c89",
    "test/java/jadx/tests/integration/others/TestInsnsBeforeThis.java": "68a918930e07eaca3b9b79a068ac86dc7389e0353358060a38f358c434609e8f",
    "test/java/jadx/tests/integration/invoke/TestConstructorWithMoves.java": "681c48dd2d07edb2c44ea68d9463c03477703fe57d00364bf87e9e8ec3d98b54",
    "main/java/jadx/core/dex/visitors/ConstructorVisitor.java": "477a662a43413d0e3dd162ee5ddcf6bb739347f678bb4a8c36687e780c74cc3b",
    "main/java/jadx/core/dex/visitors/PrepareForCodeGen.java": "51f2e498c55bcce32b26a59d60d56fedaa95bd48060c605aa6b11c757f20082b",
    "main/java/jadx/core/codegen/InsnGen.java": "c6308a71dd870d7f13bb8ac7efdb58191966cd6a5254aa11e443c95af6bafed6",
}
CLASSES = ("Base", "Child", "Factory")
EXPECTED = "a\nb\nc\n5"


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def run(command, log, *, compact=False):
    result = subprocess.run([str(part) for part in command], capture_output=True, text=True, timeout=180)
    log.parent.mkdir(parents=True, exist_ok=True)
    if compact:
        log.write_text(f"exit={result.returncode}\n"
                       f"stdout_sha256={hashlib.sha256(result.stdout.encode()).hexdigest()}\n"
                       f"stderr_sha256={hashlib.sha256(result.stderr.encode()).hexdigest()}\n")
    else:
        log.write_text("\n".join(line.rstrip() for line in
                                 (result.stdout + result.stderr).splitlines()) + "\n")
    if result.returncode:
        raise RuntimeError(f"command failed ({result.returncode}): {log}")
    return result.stdout


def compile_run(name, sources, output, temporary):
    classes = temporary / f"{name}-classes"
    classes.mkdir()
    run(["javac", "--release", "8", "-g:none", "-d", classes,
         *sources, INPUT / "Runner.java"], output / name / "javac.log")
    stdout = run(["java", "-Xverify:all", "-cp", classes, "em05.Runner"],
                 output / name / "runtime.log").strip()
    if stdout != EXPECTED:
        raise RuntimeError(f"{name} changed runtime output: {stdout!r}")
    return {"javac_exit": 0, "runtime_exit": 0, "stdout": stdout,
            "source_sha256": {p.name: digest(p) for p in sources}}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jarde", type=Path, required=True)
    parser.add_argument("--jadx", type=Path, required=True)
    parser.add_argument("--jadx-checkout", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    output = args.out.resolve()
    if output.exists() and any(output.iterdir()):
        parser.error("--out must be empty")
    output.mkdir(parents=True, exist_ok=True)
    revision = run(["git", "-C", args.jadx_checkout, "rev-parse", "HEAD"],
                   output / "jadx-revision.txt").strip()
    if revision != JADX_REV:
        raise RuntimeError("JADX checkout changed")
    for name, sha in PINS.items():
        if digest(args.jadx_checkout / "jadx-core/src" / name) != sha:
            raise RuntimeError(f"JADX pinned source changed: {name}")
    with tempfile.TemporaryDirectory(prefix="jarde-em05-") as temp_name:
        temporary = Path(temp_name)
        originals = [INPUT / f"{name}.java" for name in CLASSES]
        original = compile_run("original", originals, output, temporary)
        original_class_sha256 = {name: digest(temporary / "original-classes/em05" / f"{name}.class")
                                 for name in CLASSES}
        jar = temporary / "fixture.jar"
        run(["jar", "cf", jar, "-C", temporary / "original-classes", "."], output / "jar.log")
        jadx_dir = temporary / "jadx"
        run([args.jadx, "-d", jadx_dir, jar], output / "jadx.log")
        jadx_sources = []
        jarde_sources = []
        for name in CLASSES:
            jadx_source = output / "source" / "jadx" / "em05" / f"{name}.java"
            jadx_source.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(jadx_dir / "sources/em05" / f"{name}.java", jadx_source)
            jadx_sources.append(jadx_source)
            jarde_source = output / "source" / "jarde" / "em05" / f"{name}.java"
            jarde_source.parent.mkdir(parents=True, exist_ok=True)
            jarde_source.write_text(run(
                [args.jarde, "class-source", "--input", jar, "--class", f"em05.{name}",
                 "--policy", "plain-jar", "--release", "8", "--format", "text"],
                output / "jarde-logs" / f"{name}.log", compact=True))
            jarde_sources.append(jarde_source)
        jadx = compile_run("jadx", jadx_sources, output, temporary)
        jarde = compile_run("jarde", jarde_sources, output, temporary)
    for label in ("jadx", "jarde"):
        child = (output / "source" / label / "em05/Child.java").read_text()
        factory = (output / "source" / label / "em05/Factory.java").read_text()
        if "this(" not in child or "super(" not in child or "new " not in factory:
            raise RuntimeError(f"{label} lost one constructor syntax form")
    result = {"jadx_revision": revision, "jadx_pins": PINS,
              "jarde_cli_sha256": digest(args.jarde),
              "input_sha256": {p.name: digest(p) for p in INPUT.glob("*.java")},
              "original_class_sha256": original_class_sha256,
              "original": original, "jadx": jadx, "jarde": jarde}
    (output / "summary.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
