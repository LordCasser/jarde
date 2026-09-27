#!/usr/bin/env python3
"""Replay the fixed EM-24 Java 8 numeric-literal comparison."""

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile


HERE = Path(__file__).resolve().parent
INPUT = HERE / "input" / "em24"
JADX_REVISION = "2fb1b16386941660fda07e9017285aec40fcb37f"
TEST_HASHES = {
    "arith/TestNumbersFormat.java": "38029f8d967d5b6511fd22e66b5018ea36f2823b2461eda3eb4164a58c1e1b54",
    "arith/TestSpecialValues.java": "6d3e148e24e65795e459f52f602ef4f7ff45b40660a6e26ffed9f4029cb768be",
    "others/TestFloatValue.java": "9e17f2cf2b4998d8ec478f5f94792eea39fb3f895d8e5f3b237522d82058d175",
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
        executed = run(["java", "-Xverify:all", "-cp", classes, "em24.Runner"],
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

    with tempfile.TemporaryDirectory(prefix="jarde-em24-") as temp_name:
        temp = Path(temp_name)
        original = compile_and_run("original", INPUT / "Numbers.java", out, temp)
        if original["javac_exit"] or original.get("runtime_exit"):
            raise RuntimeError("original Java 8 source is not valid")
        jar = temp / "input.jar"
        jar_result = run(["jar", "cf", jar, "-C", temp / "original-classes", "em24/Numbers.class"],
                         out / "jar.log")
        if jar_result.returncode:
            raise RuntimeError("input jar failed")
        jadx_dir = temp / "jadx"
        jadx_result = run([args.jadx, "-d", jadx_dir, jar], out / "jadx.log")
        if jadx_result.returncode:
            raise RuntimeError("JADX failed")
        source_dir = out / "source"
        source_dir.mkdir()
        jadx_source = source_dir / "jadx-Numbers.java"
        shutil.copy2(jadx_dir / "sources/em24/Numbers.java", jadx_source)
        jarde_result = run(
            [args.jarde, "class-source", "--input", jar, "--class", "em24.Numbers",
             "--policy", "plain-jar", "--release", "8", "--evidence", "essential",
             "--format", "text"],
            out / "jarde-report.log",
        )
        if jarde_result.returncode:
            raise RuntimeError("Jarde class-source query failed")
        jarde_source = source_dir / "jarde-Numbers.java"
        jarde_source.write_text(jarde_result.stdout)
        for label, source in (("jadx", jadx_source), ("jarde", jarde_source)):
            target = temp / f"{label}-source" / "em24" / "Numbers.java"
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
