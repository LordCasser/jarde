#!/usr/bin/env python3
"""Replay the legal Java 8 invocation slice for fixed JADX EM-10."""

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
INPUT = HERE / "input"
JADX_REV = "2fb1b16386941660fda07e9017285aec40fcb37f"
PINS = {
    "test/java/jadx/tests/integration/invoke/TestInvoke1.java": "8506d42a27e8ab544c4dcd4237e9dc2080d235dcbb921e62ae6e62ec3a013c4d",
    "test/java/jadx/tests/integration/invoke/TestInvokeWithWideVars.java": "806c76dd5da29e624fd09c7d70f54f165f4f648112666d6c5822ded71cd27427",
    "test/java/jadx/tests/integration/invoke/TestInheritedStaticInvoke.java": "dedc9bb580063b696f6551bae8734e2743b8ddc7632fabb2f9d4bf757e8f7c63",
    "test/java/jadx/tests/integration/invoke/TestInvokeInCatch.java": "209dc0ae630488880ac3a36e2819d612abbc5a995ee4b8a698c22f06dd1d5b70",
    "main/java/jadx/core/dex/visitors/MethodInvokeVisitor.java": "60ee5bd34ad003d48c85bc622f7ac160f4a197acb894999140fcedf1914129ff",
    "main/java/jadx/core/codegen/InsnGen.java": "c6308a71dd870d7f13bb8ac7efdb58191966cd6a5254aa11e443c95af6bafed6",
}
CLASSES = ("InvokeBase", "InvokeFixture", "InvokeTools", "InvokeWorker")
EXPECTED = "195"
JADX_ROOT = Path("/Users/lordcasser/workspace/testzone/jadx")
JADX = JADX_ROOT / "jadx-cli/build/install/jadx/bin/jadx"


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
        raise RuntimeError(f"command failed ({result.returncode}): {log}\n{result.stderr}")
    return result.stdout


def compile_run(name, sources, output, temporary):
    classes = temporary / f"{name}-classes"
    classes.mkdir()
    run(["javac", "--release", "8", "-g:none", "-d", classes, *sources],
        output / name / "javac.log")
    stdout = run(["java", "-Xverify:all", "-cp", classes, "em10.InvokeFixture"],
                 output / name / "runtime.log").strip()
    if stdout != EXPECTED:
        raise RuntimeError(f"{name} changed runtime output: {stdout!r}, expected {EXPECTED!r}")
    return {"javac_exit": 0, "runtime_exit": 0, "stdout": stdout,
            "source_sha256": {p.name: digest(p) for p in sources}}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jarde", type=Path, required=True)
    parser.add_argument("--jadx-checkout", type=Path, default=JADX_ROOT)
    parser.add_argument("--jadx", type=Path, default=JADX)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    output = args.out.resolve()
    if output.exists() and any(output.iterdir()):
        parser.error("--out must be empty")
    output.mkdir(parents=True, exist_ok=True)
    revision = run(["git", "-C", args.jadx_checkout, "rev-parse", "HEAD"],
                   output / "jadx-revision.txt").strip()
    if revision != JADX_REV:
        raise RuntimeError(f"JADX checkout changed: {revision}")
    for name, sha in PINS.items():
        if digest(args.jadx_checkout / "jadx-core/src" / name) != sha:
            raise RuntimeError(f"JADX pinned source changed: {name}")
    with tempfile.TemporaryDirectory(prefix="jarde-em10-") as temp_name:
        temporary = Path(temp_name)
        originals = [INPUT / f"{name}.java" for name in CLASSES]
        original = compile_run("original", originals, output, temporary)
        original_class_sha256 = {
            name: digest(temporary / "original-classes/em10" / f"{name}.class")
            for name in CLASSES
        }
        jar = temporary / "fixture.jar"
        run(["jar", "cf", jar, "-C", temporary / "original-classes", "."], output / "jar.log")
        jadx_dir = temporary / "jadx"
        run([args.jadx, "-d", jadx_dir, jar], output / "jadx.log")
        jadx_sources = []
        jarde_sources = []
        for name in CLASSES:
            jadx_source = output / "source" / "jadx" / "em10" / f"{name}.java"
            jadx_source.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(jadx_dir / "sources/em10" / f"{name}.java", jadx_source)
            jadx_sources.append(jadx_source)
            jarde_source = output / "source" / "jarde" / "em10" / f"{name}.java"
            jarde_source.parent.mkdir(parents=True, exist_ok=True)
            jarde_source.write_text(run(
                [args.jarde, "class-source", "--input", jar, "--class", f"em10.{name}",
                 "--policy", "plain-jar", "--release", "8", "--format", "text"],
                output / "jarde-logs" / f"{name}.log", compact=True))
            jarde_sources.append(jarde_source)
        jadx = compile_run("jadx", jadx_sources, output, temporary)
        jarde = compile_run("jarde", jarde_sources, output, temporary)
    for label in ("jadx", "jarde"):
        fixture = (output / "source" / label / "em10/InvokeFixture.java").read_text()
        expected_forms = ("receiver().combine(", "InvokeTools.combine(",
                          "inherited(", "catch (", "InvokeTools.caught(")
        missing = [form for form in expected_forms if form not in fixture]
        if missing:
            raise RuntimeError(f"{label} source lost invocation forms: {missing}")
    result = {"jadx_revision": revision, "jadx_pins": PINS,
              "jarde_cli_sha256": digest(args.jarde),
              "input_sha256": {p.name: digest(p) for p in INPUT.glob("*.java")},
              "original_class_sha256": original_class_sha256,
              "original": original, "jadx": jadx, "jarde": jarde}
    (output / "summary.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
