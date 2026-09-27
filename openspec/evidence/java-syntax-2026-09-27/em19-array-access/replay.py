#!/usr/bin/env python3
"""Replay the bounded EM-19 array-access Java 8 comparison."""

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile


HERE = Path(__file__).resolve().parent
INPUT = HERE / "input" / "em19"
JADX_REVISION = "2fb1b16386941660fda07e9017285aec40fcb37f"
TEST_HASHES = {
    "arrays/TestArrays.java": "ca3b832fc63f4674802d56fed16915f95c57f7d5b26f9f72451830c451bb9284",
    "code/TestArrayAccessReorder.java": "ae82af6f1838d8d0d858fd7493ad69392531a8ea0db61a0b07d33ba83a4978bc",
}
EXPECTED = "1:5\nAIOOBE\n2\n[-15, -10, -5]"


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
    executed = run(["java", "-Xverify:all", "-cp", classes, "em19.Runner"], out / label / "runtime.log")
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

    with tempfile.TemporaryDirectory(prefix="jarde-em19-") as temp_name:
        temp = Path(temp_name)
        original = compile_and_run("original", INPUT / "Access.java", out, temp)
        jar = temp / "input.jar"
        if run(["jar", "cf", jar, "-C", temp / "original-classes", "."], out / "jar.log").returncode:
            raise RuntimeError("input jar failed")
        source_dir = out / "source"
        source_dir.mkdir()
        jadx_root = temp / "jadx"
        if run([args.jadx, "-d", jadx_root, jar], out / "jadx.log").returncode:
            raise RuntimeError("JADX failed")
        jadx_source = source_dir / "jadx-Access.java"
        shutil.copy2(jadx_root / "sources/em19/Access.java", jadx_source)
        jarde_result = run(
            [args.jarde, "class-source", "--input", jar, "--class", "em19.Access",
             "--policy", "plain-jar", "--release", "8", "--evidence", "essential", "--format", "text"],
            out / "jarde-report.log",
        )
        if jarde_result.returncode:
            raise RuntimeError("Jarde class-source failed")
        jarde_source = source_dir / "jarde-Access.java"
        jarde_source.write_text(jarde_result.stdout)
        for expected in ("return local1[arg0]", "return local1.length", "arg0[local3]",
                         "local3 = local3 + 1", "local2[local4] = local6 * 5"):
            if expected not in jarde_result.stdout:
                raise RuntimeError(f"Jarde array source shape missing: {expected}")
        for label, source in (("jadx", jadx_source), ("jarde", jarde_source)):
            target = temp / f"{label}-source" / "em19" / "Access.java"
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
