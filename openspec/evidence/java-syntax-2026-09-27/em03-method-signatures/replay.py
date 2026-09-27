#!/usr/bin/env python3
"""Replay EM-03 method declaration / MethodParameters on fixed Java 8 sources."""

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile


HERE = Path(__file__).resolve().parent
INPUT = HERE / "input" / "em03"
JADX_REVISION = "2fb1b16386941660fda07e9017285aec40fcb37f"
TEST_HASHES = {
    "others/TestMethodParametersAttribute.java": "7d76c5f1f59850dd24d38c600c8c2c3e5ca050eebacf1c53ae0e35b8d065b7ed",
    "others/TestThrows.java": "f6349d4ef0217ab3003b5c18510030ad8f12b66e66e6a40fc43adf6fcdd7ee5f",
    "others/TestMissingExceptions.java": "3938383630e374f4a2db3b28413d5a7d965e7b2b809c883b6fdbec3abbcd83cc",
    "others/TestInvalidExceptions.java": "c7336423737a003022b8fbfe71d05b10023e23332cd6e2f687bd66dfc9d30637",
    "others/TestIncorrectMethodSignature.java": "65b3d33fa903a77e18bfe0dd004e16b478cb9dbdb6ece7d151a1a7bdc6ddef7e",
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
        ["javac", "--release", "8", "-g:none", "-parameters", "-d", classes,
         source, INPUT / "Runner.java"],
        out / label / "javac.log",
    )
    result = {"javac_exit": compiled.returncode, "source_sha256": digest(source)}
    if compiled.returncode == 0:
        executed = run(["java", "-Xverify:all", "-cp", classes, "em03.Runner"],
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

    with tempfile.TemporaryDirectory(prefix="jarde-em03-") as temp_name:
        temp = Path(temp_name)
        original = compile_and_run("original", INPUT / "Signatures.java", out, temp)
        if original["javac_exit"] or original.get("runtime_exit"):
            raise RuntimeError("original Java 8 source is invalid")
        run(["javap", "-classpath", temp / "original-classes", "-v", "em03.Signatures"],
            out / "javap-Signatures.txt")
        jar = temp / "input.jar"
        if run(["jar", "cf", jar, "-C", temp / "original-classes", "em03/Signatures.class"],
               out / "jar.log").returncode:
            raise RuntimeError("input jar failed")
        jadx_dir = temp / "jadx"
        if run([args.jadx, "-d", jadx_dir, jar], out / "jadx.log").returncode:
            raise RuntimeError("JADX failed")
        source_dir = out / "source"
        source_dir.mkdir()
        jadx_source = source_dir / "jadx-Signatures.java"
        shutil.copy2(jadx_dir / "sources/em03/Signatures.java", jadx_source)
        jarde_result = run(
            [args.jarde, "class-source", "--input", jar, "--class", "em03.Signatures",
             "--policy", "plain-jar", "--release", "8", "--evidence", "essential",
             "--format", "text"],
            out / "jarde-report.log",
        )
        if jarde_result.returncode:
            raise RuntimeError("Jarde class-source query failed")
        jarde_source = source_dir / "jarde-Signatures.java"
        jarde_source.write_text(jarde_result.stdout)
        results = {}
        for label, source in (("jadx", jadx_source), ("jarde", jarde_source)):
            target = temp / f"{label}-source" / "em03" / "Signatures.java"
            target.parent.mkdir(parents=True)
            shutil.copy2(source, target)
            results[label] = compile_and_run(label, target, out, temp)
    summary = {
        "jadx_revision": JADX_REVISION,
        "jadx_test_sha256": TEST_HASHES,
        "jarde_cli_sha256": digest(args.jarde.resolve()),
        "input_sha256": {path.name: digest(path) for path in sorted(INPUT.glob("*.java"))},
        "original": original,
        **results,
    }
    (out / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
