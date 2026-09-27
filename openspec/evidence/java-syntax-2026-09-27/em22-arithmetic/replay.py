#!/usr/bin/env python3
"""Replay the fixed EM-22 arithmetic Java 8 comparison."""

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile


HERE = Path(__file__).resolve().parent
INPUT = HERE / "input" / "em22"
JADX_REVISION = "2fb1b16386941660fda07e9017285aec40fcb37f"
TEST_HASHES = {
    "arith/TestArith2.java": "2beeb0b5f24bf9328915bf5291430cd067fcc88ce2da3cd1f07c21f1e4d58eaf",
    "arith/TestArithNot.java": "04757ed4b06cd7f2b7ccd1bf9a4fe0d2b13a02f4c96f956b0025b5d4c7d62493",
    "arith/TestXor.java": "fd2577fe828a9f73b21a7022e8177245710ef98ceec3317dacc099e4c1c3f2b3",
    "others/TestRedundantBrackets.java": "ac67d1c9a8a780527faeb77d12db87d59059be5ce894d9e43d7fa3740d1f81f1",
}
EXPECTED = "18:6\n14:5\ntrue:false\n-1:-1\nfalse:true\ntrue:2\nfalse:3\ntrue:4"


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
    compiled = run(
        ["javac", "--release", "8", "-g:none", "-d", classes, source, INPUT / "Runner.java"],
        out / label / "javac.log",
    )
    if compiled.returncode:
        raise RuntimeError(f"{label}: Java 8 compilation failed")
    executed = run(["java", "-Xverify:all", "-cp", classes, "em22.Runner"], out / label / "runtime.log")
    if executed.returncode or executed.stdout.strip() != EXPECTED:
        raise RuntimeError(f"{label}: verifier/runtime mismatch")
    return {"javac_exit": compiled.returncode, "runtime_exit": executed.returncode,
            "stdout": executed.stdout.strip(), "source_sha256": digest(source)}


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
    for relative, expected_hash in TEST_HASHES.items():
        source = args.jadx_checkout / "jadx-core/src/test/java/jadx/tests/integration" / relative
        if digest(source) != expected_hash:
            raise RuntimeError(f"JADX test changed: {relative}")
    if run(["javac", "-version"], out / "javac-version.txt").returncode:
        raise RuntimeError("javac unavailable")
    if run(["java", "-version"], out / "java-version.txt").returncode:
        raise RuntimeError("java unavailable")

    with tempfile.TemporaryDirectory(prefix="jarde-em22-") as temp_name:
        temp = Path(temp_name)
        original = compile_and_run("original", INPUT / "Arithmetic.java", out, temp)
        jar = temp / "input.jar"
        if run(["jar", "cf", jar, "-C", temp / "original-classes", "."], out / "jar.log").returncode:
            raise RuntimeError("input jar failed")
        source_dir = out / "source"
        source_dir.mkdir()
        jadx_root = temp / "jadx"
        if run([args.jadx, "-d", jadx_root, jar], out / "jadx.log").returncode:
            raise RuntimeError("JADX failed")
        jadx_source = source_dir / "jadx-Arithmetic.java"
        shutil.copy2(jadx_root / "sources/em22/Arithmetic.java", jadx_source)
        jarde_result = run(
            [args.jarde, "class-source", "--input", jar, "--class", "em22.Arithmetic",
             "--policy", "plain-jar", "--release", "8", "--evidence", "essential", "--format", "text"],
            out / "jarde-report.log",
        )
        if jarde_result.returncode:
            raise RuntimeError("Jarde class-source failed")
        jarde_source = source_dir / "jarde-Arithmetic.java"
        jarde_source.write_text(jarde_result.stdout)
        for expected in ("return (arg1 + 2) * 3", "return arg1 - (arg2 - arg3)",
                         "return arg1 / (arg2 / arg3)", "return this.left() | this.right()",
                         "return !arg1", "return !this.left()",
                         "return this.left()", "return arg1 ^ -1L"):
            if expected not in jarde_result.stdout:
                raise RuntimeError(f"Jarde arithmetic source shape missing: {expected}")
        for redundant in ("return arg1 ^ true", "return this.left() ^ true",
                          "return this.left() ^ false"):
            if redundant in jarde_result.stdout:
                raise RuntimeError(f"Jarde retained a proved redundant XOR: {redundant}")
        jadx_text = jadx_source.read_text()
        for expected in ("return (i + 2) * 3", "return i - (i2 - i3)",
                         "return i / (i2 / i3)", "return left() | right()",
                         "return !z", "return !left()", "return left()", "return j ^ (-1)"):
            if expected not in jadx_text:
                raise RuntimeError(f"JADX arithmetic source shape missing: {expected}")
        for label, source in (("jadx", jadx_source), ("jarde", jarde_source)):
            target = temp / f"{label}-source" / "em22" / "Arithmetic.java"
            target.parent.mkdir(parents=True)
            shutil.copy2(source, target)
            result = compile_and_run(label, target, out, temp)
            if label == "jadx":
                jadx = result
            else:
                jarde = result
    summary = {
        "jadx_revision": JADX_REVISION, "jadx_test_sha256": TEST_HASHES,
        "jarde_cli_sha256": digest(args.jarde.resolve()),
        "input_sha256": {p.name: digest(p) for p in sorted(INPUT.glob("*.java"))},
        "original": original, "jadx": jadx, "jarde": jarde,
    }
    (out / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
