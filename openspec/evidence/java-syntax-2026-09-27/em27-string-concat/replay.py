#!/usr/bin/env python3
"""Replay the fixed EM-27 Java 8 string-concatenation comparison."""

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile


HERE = Path(__file__).resolve().parent
INPUT = HERE / "input" / "em27"
JADX_REVISION = "2fb1b16386941660fda07e9017285aec40fcb37f"
TEST_HASHES = {
    "others/TestConstStringConcat.java": "27e33208d33e2e280ce0bf92cf0f630b68f4d3adf5a8aa11b19097748a5d5e95",
    "others/TestStringBuilderElimination.java": "228824ae3c14a5655782c89778011242a9f060fd260060c0adb66272ee5d4f2f",
    "others/TestStringBuilderElimination4Neg.java": "a3c1294133a9b6cea744d47dc7cd077fcad1768c6522786bb78b08914793a42d",
    "others/TestStringConcatJava11.java": "e6e6b1e81df42f59a7f58ccaea2945f1e3d6047f92790bd930d6629c3a21d7bc",
    "others/TestStringConcatWithoutResult.java": "c959f32583c65cdcaf74a4f7d1f769aaecae2e8875d5fd891826e43459eabb80",
    "others/TestStringConstructor.java": "6be750f5fb0311c92eafd17be5bce7bdff56d1d8ec2979fb226218d34d0b7a22",
    "others/TestIssue13a.java": "28bfa600c96bdd85113fd9b760460d471d13ac256f7885b6ee0c7e50aaafbdf1",
}


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
    result = {"javac_exit": compiled.returncode, "source_sha256": digest(source)}
    if compiled.returncode == 0:
        executed = run(["java", "-Xverify:all", "-cp", classes, "em27.Runner"],
                       out / label / "runtime.log")
        result.update({"runtime_exit": executed.returncode, "stdout": executed.stdout.strip()})
    return result


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
    for relative, expected in TEST_HASHES.items():
        source = args.jadx_checkout / "jadx-core/src/test/java/jadx/tests/integration" / relative
        if digest(source) != expected:
            raise RuntimeError(f"JADX test changed: {relative}")
    run(["javac", "-version"], out / "javac-version.txt")
    run(["java", "-version"], out / "java-version.txt")

    with tempfile.TemporaryDirectory(prefix="jarde-em27-") as temp_name:
        temp = Path(temp_name)
        original = compile_and_run("original", INPUT / "Concat.java", out, temp)
        if original["javac_exit"] or original.get("runtime_exit"):
            raise RuntimeError("original Java 8 source is not valid")
        run(["javap", "-classpath", temp / "original-classes", "-c", "-p", "em27.Concat"],
            out / "javap-Concat.txt")
        jar = temp / "input.jar"
        jar_result = run(["jar", "cf", jar, "-C", temp / "original-classes", "em27/Concat.class"],
                         out / "jar.log")
        if jar_result.returncode:
            raise RuntimeError("input jar failed")
        jadx_dir = temp / "jadx"
        jadx_result = run([args.jadx, "-d", jadx_dir, jar], out / "jadx.log")
        if jadx_result.returncode:
            raise RuntimeError("JADX failed")
        source_dir = out / "source"
        source_dir.mkdir()
        jadx_source = source_dir / "jadx-Concat.java"
        shutil.copy2(jadx_dir / "sources/em27/Concat.java", jadx_source)
        jarde_result = run(
            [args.jarde, "class-source", "--input", jar, "--class", "em27.Concat",
             "--policy", "plain-jar", "--release", "8", "--evidence", "essential",
             "--format", "text"],
            out / "jarde-report.log",
        )
        if jarde_result.returncode:
            raise RuntimeError("Jarde class-source query failed")
        jarde_source = source_dir / "jarde-Concat.java"
        jarde_source.write_text(jarde_result.stdout)
        for label, source in (("jadx", jadx_source), ("jarde", jarde_source)):
            target = temp / f"{label}-source" / "em27" / "Concat.java"
            target.parent.mkdir(parents=True)
            shutil.copy2(source, target)
            if label == "jadx":
                jadx = compile_and_run(label, target, out, temp)
            else:
                jarde = compile_and_run(label, target, out, temp)
    summary = {
        "jadx_revision": JADX_REVISION,
        "jadx_test_sha256": TEST_HASHES,
        "jarde_cli_sha256": digest(args.jarde.resolve()),
        "input_sha256": {path.name: digest(path) for path in sorted(INPUT.glob("*.java"))},
        "original": original,
        "jadx": jadx,
        "jarde": jarde,
    }
    (out / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
