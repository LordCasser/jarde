#!/usr/bin/env python3
"""Replay the bounded EM-18 Java 8 array-initializer comparison."""

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile


HERE = Path(__file__).resolve().parent
INPUT = HERE / "input" / "em18"
JADX_REVISION = "2fb1b16386941660fda07e9017285aec40fcb37f"
TEST_HASHES = {
    "arrays/TestArrayFill.java": "b256b87e7467f17f149193e1e9708ae1fd2d1e6e93303ca6ecd52329812246ce",
    "arrays/TestArrayFill2.java": "2aa4e56ea67144f2a1fb72048f9e5f430c7f351a1e4b1efe2e9d34ab9f715144",
    "arrays/TestArrayFillNegative.java": "c1a10209342aba4e3a1fda64e1eee30309b80b613c0e6b3485fa169a595b80a1",
    "types/TestArrayTypes.java": "8e7ee5ce0e376136a99e29a7fd17039b6906d4bfb2c7f22f0aad91b2f37a14b5",
}
EXPECTED = "[1, 2, 3]\n[1, 3, 2]\n[1, 2, 6]\n[1, 2, 3]\n1"


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def run(command, log):
    result = subprocess.run([str(part) for part in command], capture_output=True, text=True, timeout=180)
    log.parent.mkdir(parents=True, exist_ok=True)
    lines = (result.stdout + result.stderr).splitlines()
    log.write_text("\n".join(line.rstrip() for line in lines) + ("\n" if lines else ""))
    return result


def compile_and_run(label, array_source, out, temp):
    classes = temp / f"{label}-classes"
    classes.mkdir()
    compile_result = run(
        ["javac", "--release", "8", "-g:none", "-d", classes, array_source, INPUT / "Runner.java"],
        out / label / "javac.log",
    )
    if compile_result.returncode:
        raise RuntimeError(f"{label}: Java 8 compilation failed")
    execution = run(["java", "-Xverify:all", "-cp", classes, "em18.Runner"], out / label / "runtime.log")
    if execution.returncode or execution.stdout.strip() != EXPECTED:
        raise RuntimeError(f"{label}: verifier/runtime mismatch")
    return {"javac_exit": compile_result.returncode, "runtime_exit": execution.returncode,
            "stdout": execution.stdout.strip(), "source_sha256": digest(array_source)}


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
        source = (args.jadx_checkout / "jadx-core/src/test/java/jadx/tests/integration" / relative)
        if digest(source) != expected_hash:
            raise RuntimeError(f"JADX test changed: {relative}")
    javac_version = run(["javac", "-version"], out / "javac-version.txt")
    java_version = run(["java", "-version"], out / "java-version.txt")
    if javac_version.returncode or java_version.returncode:
        raise RuntimeError("Java toolchain unavailable")

    with tempfile.TemporaryDirectory(prefix="jarde-em18-") as temp_name:
        temp = Path(temp_name)
        original = compile_and_run("original", INPUT / "Arrays.java", out, temp)
        jar = temp / "input.jar"
        if run(["jar", "cf", jar, "-C", temp / "original-classes", "."], out / "jar.log").returncode:
            raise RuntimeError("input jar failed")
        source_dir = out / "source"
        source_dir.mkdir()
        jadx_root = temp / "jadx"
        if run([args.jadx, "-d", jadx_root, jar], out / "jadx.log").returncode:
            raise RuntimeError("JADX failed")
        jadx_source = source_dir / "jadx-Arrays.java"
        shutil.copy2(jadx_root / "sources/em18/Arrays.java", jadx_source)
        jarde_result = run(
            [args.jarde, "class-source", "--input", jar, "--class", "em18.Arrays",
             "--policy", "plain-jar", "--release", "8", "--evidence", "essential", "--format", "text"],
            out / "jarde-report.log",
        )
        if jarde_result.returncode:
            raise RuntimeError("Jarde class-source failed")
        jarde_source = source_dir / "jarde-Arrays.java"
        jarde_source.write_text(jarde_result.stdout)
        jadx_text = jadx_source.read_text()
        jarde_text = jarde_source.read_text()
        required = (
            'new java.lang.String[]{"1", "2", "3"}',
            "new int[]{1, arg0 + 1, 2}",
            "new int[]{1, arg0++, arg0 * 2}",
            "local0[1] = local0[0] + 1",
            "new java.lang.Object[]{arg0}",
        )
        if any(part not in jarde_text for part in required):
            raise RuntimeError("Jarde positive or negative source shape changed")
        if "new int[]{1, i, (i + 1) * 2}" not in jadx_text or "i++" in jadx_text:
            raise RuntimeError("fixed JADX postfix source boundary changed")
        # javac requires the physical public-class filename, not the named evidence snapshot.
        for label, source in (("jadx", jadx_source), ("jarde", jarde_source)):
            target = temp / f"{label}-source" / "em18" / "Arrays.java"
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
        "original": original,
        "jadx": jadx,
        "jarde": jarde,
        "postfix_source": {"jadx": "new int[]{1, i, (i + 1) * 2}",
                           "jarde": "new int[]{1, arg0++, arg0 * 2}"},
    }
    (out / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
