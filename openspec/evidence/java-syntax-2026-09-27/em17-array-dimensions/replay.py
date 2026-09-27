#!/usr/bin/env python3
"""Replay the bounded EM-17 Java 8 array-dimension comparison."""

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile


HERE = Path(__file__).resolve().parent
INPUT = HERE / "input" / "em17"
JADX_REVISION = "2fb1b16386941660fda07e9017285aec40fcb37f"
TEST_HASHES = {
    "TestNewArrayOfArrays.java": "0e6328792cc178aab5dc223fe0675a0a6f3bfbe7aedfaea94ca64e661bbe7676",
    "TestMultiDimArrayFill.java": "1c90d26dcecf1db798deb0fbab61cf84a9ee0bf704caf3152af64e6aef6d0b35",
    "TestArrays3.java": "3ebb5aa6d4dea2c67eb49db29611021583273637c905880f6779e504419d6b4b",
    "TestArrays4.java": "daa5acb3dc6bcdfdcb9283f8717a34e61243090bd5c99509f41d6221735add67",
}
EXPECTED = "2:true\n3:true\n4:true\n2:3\n[[1], [4, 5], []]\ntrue\n2"


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def run(command, log):
    result = subprocess.run([str(part) for part in command], capture_output=True, text=True, timeout=180)
    log.parent.mkdir(parents=True, exist_ok=True)
    lines = (result.stdout + result.stderr).splitlines()
    log.write_text("\n".join(line.rstrip() for line in lines) + ("\n" if lines else ""))
    return result


def compile_and_run(label, source, out, temp):
    classes = temp / f"{label}-classes"
    classes.mkdir()
    compile_result = run(
        ["javac", "--release", "8", "-g:none", "-d", classes, source, INPUT / "Runner.java"],
        out / label / "javac.log",
    )
    if compile_result.returncode:
        raise RuntimeError(f"{label}: Java 8 compilation failed")
    execution = run(["java", "-Xverify:all", "-cp", classes, "em17.Runner"], out / label / "runtime.log")
    if execution.returncode or execution.stdout.strip() != EXPECTED:
        raise RuntimeError(f"{label}: verifier/runtime mismatch")
    return {"javac_exit": compile_result.returncode, "runtime_exit": execution.returncode,
            "stdout": execution.stdout.strip(), "source_sha256": digest(source)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jarde", type=Path, required=True)
    parser.add_argument("--jadx", type=Path, required=True)
    parser.add_argument("--jadx-checkout", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    out = args.out.resolve()
    if out.exists() and any(out.iterdir()):
        parser.error("--out must be absent or empty")
    out.mkdir(parents=True, exist_ok=True)
    revision = run(["git", "-C", args.jadx_checkout, "rev-parse", "HEAD"], out / "jadx-revision.txt")
    if revision.returncode or revision.stdout.strip() != JADX_REVISION:
        raise RuntimeError("JADX checkout revision changed")
    for name, expected_hash in TEST_HASHES.items():
        source = args.jadx_checkout / "jadx-core/src/test/java/jadx/tests/integration/arrays" / name
        if digest(source) != expected_hash:
            raise RuntimeError(f"JADX test changed: {name}")
    if run(["javac", "-version"], out / "javac-version.txt").returncode:
        raise RuntimeError("javac unavailable")
    if run(["java", "-version"], out / "java-version.txt").returncode:
        raise RuntimeError("java unavailable")

    with tempfile.TemporaryDirectory(prefix="jarde-em17-") as temp_name:
        temp = Path(temp_name)
        original = compile_and_run("original", INPUT / "Shapes.java", out, temp)
        jar = temp / "input.jar"
        if run(["jar", "cf", jar, "-C", temp / "original-classes", "."], out / "jar.log").returncode:
            raise RuntimeError("input jar failed")
        source_dir = out / "source"
        source_dir.mkdir()
        jadx_root = temp / "jadx"
        if run([args.jadx, "-d", jadx_root, jar], out / "jadx.log").returncode:
            raise RuntimeError("JADX failed")
        jadx_source = source_dir / "jadx-Shapes.java"
        shutil.copy2(jadx_root / "sources/em17/Shapes.java", jadx_source)
        jarde_result = run(
            [args.jarde, "class-source", "--input", jar, "--class", "em17.Shapes",
             "--policy", "plain-jar", "--release", "8", "--evidence", "essential", "--format", "text"],
            out / "jarde-report.log",
        )
        if jarde_result.returncode:
            raise RuntimeError("Jarde class-source failed")
        jarde_source = source_dir / "jarde-Shapes.java"
        jarde_source.write_text(jarde_result.stdout)
        for expected in ("new long[arg0][]", "new java.lang.String[arg0][]", "new int[arg0][][]",
                         "new int[arg0][arg1]", "new int[][]{new int[]{1}, new int[]{arg0, arg1}, new int[0]}",
                         "new java.lang.Object[]{arg0}", "char[] local2 = toChars(arg1)"):
            if expected not in jarde_result.stdout:
                raise RuntimeError(f"Jarde source shape missing: {expected}")
        for label, source in (("jadx", jadx_source), ("jarde", jarde_source)):
            target = temp / f"{label}-source" / "em17" / "Shapes.java"
            target.parent.mkdir(parents=True)
            shutil.copy2(source, target)
            result = compile_and_run(label, target, out, temp)
            if label == "jadx":
                jadx = result
            else:
                jarde = result
    summary = {
        "jadx_revision": JADX_REVISION,
        "jadx_test_sha256": TEST_HASHES,
        "jarde_cli_sha256": digest(args.jarde.resolve()),
        "input_sha256": {p.name: digest(p) for p in sorted(INPUT.glob("*.java"))},
        "original": original, "jadx": jadx, "jarde": jarde,
    }
    (out / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
