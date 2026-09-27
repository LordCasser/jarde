#!/usr/bin/env python3
"""Replay the EM-07 category-2 field-assignment result in Java 8."""

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile


HERE = Path(__file__).resolve().parent
INPUT = HERE / "input" / "em07"
JADX_REVISION = "2fb1b16386941660fda07e9017285aec40fcb37f"
JADX_TEST_HASH = "ea4e2e910465126c71a887c1f21886f8d18bc39bab6237cbfc03c546fa4e1335"
EXPECTED = "4294967297:4294967297\n-1:-1"


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
        return {"javac_exit": compiled.returncode, "runtime_exit": None, "stdout": None,
                "source_sha256": digest(source)}
    executed = run(["java", "-Xverify:all", "-cp", classes, "em07.Runner"], out / label / "runtime.log")
    return {"javac_exit": compiled.returncode, "runtime_exit": executed.returncode,
            "stdout": executed.stdout.strip(), "source_sha256": digest(source)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=("baseline", "fixed"))
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
    reference = args.jadx_checkout / "jadx-core/src/test/java/jadx/tests/integration/jbc/TestDup2x1.java"
    if digest(reference) != JADX_TEST_HASH:
        raise RuntimeError("fixed JADX TestDup2x1 changed")
    if run(["javac", "-version"], out / "javac-version.txt").returncode:
        raise RuntimeError("javac unavailable")
    if run(["java", "-version"], out / "java-version.txt").returncode:
        raise RuntimeError("java unavailable")

    with tempfile.TemporaryDirectory(prefix="jarde-em07-") as temp_name:
        temp = Path(temp_name)
        original = compile_and_run("original", INPUT / "Assignment.java", out, temp)
        if original["javac_exit"] or original["runtime_exit"] or original["stdout"] != EXPECTED:
            raise RuntimeError("original Java 8 source failed")
        bytecode = run(["javap", "-c", "-p", "-classpath", temp / "original-classes",
                        "em07.Assignment"], out / "javap-Assignment.txt")
        if bytecode.returncode or "dup2_x1" not in bytecode.stdout:
            raise RuntimeError("Java 8 compiler did not produce dup2_x1")
        original_class = temp / "original-classes" / "em07" / "Assignment.class"
        class_snapshot = out / "source" / "Assignment.class"
        class_snapshot.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(original_class, class_snapshot)
        jar = temp / "input.jar"
        if run(["jar", "cf", jar, "-C", temp / "original-classes", "."], out / "jar.log").returncode:
            raise RuntimeError("input jar failed")
        jadx_root = temp / "jadx"
        if run([args.jadx, "-d", jadx_root, jar], out / "jadx.log").returncode:
            raise RuntimeError("JADX failed")
        source_dir = out / "source"
        source_dir.mkdir(exist_ok=True)
        for source in sorted(INPUT.glob("*.java")):
            shutil.copy2(source, source_dir / source.name)
        jadx_source = source_dir / "jadx-Assignment.java"
        shutil.copy2(jadx_root / "sources/em07/Assignment.java", jadx_source)
        if "this.value = j;" not in jadx_source.read_text() or "return j;" not in jadx_source.read_text():
            raise RuntimeError("fixed JADX assignment source changed")
        jarde_result = run(
            [args.jarde, "class-source", "--input", jar, "--class", "em07.Assignment",
             "--policy", "plain-jar", "--release", "8", "--evidence", "essential", "--format", "text"],
            out / "jarde-report.log",
        )
        if jarde_result.returncode:
            raise RuntimeError("Jarde class-source failed")
        jarde_source = source_dir / "jarde-Assignment.java"
        jarde_source.write_text(jarde_result.stdout)
        for label, source in (("jadx", jadx_source), ("jarde", jarde_source)):
            target = temp / f"{label}-source" / "em07" / "Assignment.java"
            target.parent.mkdir(parents=True)
            shutil.copy2(source, target)
            result = compile_and_run(label, target, out, temp)
            if label == "jadx":
                jadx = result
            else:
                jarde = result
        if jadx["javac_exit"] or jadx["runtime_exit"] or jadx["stdout"] != EXPECTED:
            raise RuntimeError("fixed JADX complete source failed")
        if args.mode == "baseline":
            if jarde["javac_exit"] == 0 or "not recovered" not in jarde_result.stdout:
                raise RuntimeError("baseline Jarde field-assignment gap disappeared")
        elif (jarde["javac_exit"] or jarde["runtime_exit"] or jarde["stdout"] != EXPECTED
              or "this.value = arg1;" not in jarde_result.stdout
              or "return arg1;" not in jarde_result.stdout):
            raise RuntimeError("fixed Jarde assignment recovery failed")
    summary = {
        "mode": args.mode, "jadx_revision": JADX_REVISION,
        "jadx_test_sha256": JADX_TEST_HASH,
        "jarde_cli_sha256": digest(args.jarde.resolve()),
        "input_sha256": {p.name: digest(p) for p in sorted(INPUT.glob("*.java"))},
        "original_class_sha256": digest(out / "source" / "Assignment.class"),
        "original": original, "jadx": jadx, "jarde": jarde,
    }
    (out / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
