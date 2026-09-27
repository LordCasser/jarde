#!/usr/bin/env python3
"""Replay the fixed EM-21 this-alias Java 8 comparison."""

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile


HERE = Path(__file__).resolve().parent
INPUT = HERE / "input" / "em21"
JADX_REVISION = "2fb1b16386941660fda07e9017285aec40fcb37f"
TEST_HASHES = {
    "TestInlineThis.java": "997a6fcd6716a951254b8ece04cb8716cfdba0cd896bd83969b9203eb563816f",
    "TestInlineThis2.java": "f0f5de2c25150b49cdd8558f74853386384204aea069edf4d3d7b46c5a4645a8",
    "TestDontInlineThis.java": "91f53d32e47b4fe75016edc9825c1144d4890239bac277c0688301f8c1ed8395",
    "TestRedundantThis.java": "2ed9975cd95aade76f9cbb12abf7599a0df3f255a7ac69144ae5eab26827c6ba",
}
EXPECTED = "123:1\n123:2\ntrue:3\ntrue:1"


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
    executed = run(["java", "-Xverify:all", "-cp", classes, "em21.Runner"], out / label / "runtime.log")
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
        source = args.jadx_checkout / "jadx-core/src/test/java/jadx/tests/integration/usethis" / relative
        if digest(source) != expected_hash:
            raise RuntimeError(f"JADX test changed: {relative}")
    if run(["javac", "-version"], out / "javac-version.txt").returncode:
        raise RuntimeError("javac unavailable")
    if run(["java", "-version"], out / "java-version.txt").returncode:
        raise RuntimeError("java unavailable")

    with tempfile.TemporaryDirectory(prefix="jarde-em21-") as temp_name:
        temp = Path(temp_name)
        original = compile_and_run("original", INPUT / "ThisUse.java", out, temp)
        jar = temp / "input.jar"
        if run(["jar", "cf", jar, "-C", temp / "original-classes", "."], out / "jar.log").returncode:
            raise RuntimeError("input jar failed")
        source_dir = out / "source"
        source_dir.mkdir()
        jadx_root = temp / "jadx"
        if run([args.jadx, "-d", jadx_root, jar], out / "jadx.log").returncode:
            raise RuntimeError("JADX failed")
        jadx_source = source_dir / "jadx-ThisUse.java"
        shutil.copy2(jadx_root / "sources/em21/ThisUse.java", jadx_source)
        jarde_result = run(
            [args.jarde, "class-source", "--input", jar, "--class", "em21.ThisUse",
             "--policy", "plain-jar", "--release", "8", "--evidence", "essential", "--format", "text"],
            out / "jarde-report.log",
        )
        if jarde_result.returncode:
            raise RuntimeError("Jarde class-source failed")
        jarde_source = source_dir / "jarde-ThisUse.java"
        jarde_source.write_text(jarde_result.stdout)
        for expected in ("local1 = this", "local1.touch()", "local1.field = 123",
                         "local1 = new em21.ThisUse()", "return local1"):
            if expected not in jarde_result.stdout:
                raise RuntimeError(f"Jarde alias source shape missing: {expected}")
        jadx_text = jadx_source.read_text()
        for expected in ("Objects.isNull(this)", "this.field = 123", "thisUse = this",
                         "thisUse = new ThisUse()"):
            if expected not in jadx_text:
                raise RuntimeError(f"JADX this source shape missing: {expected}")
        for label, source in (("jadx", jadx_source), ("jarde", jarde_source)):
            target = temp / f"{label}-source" / "em21" / "ThisUse.java"
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
