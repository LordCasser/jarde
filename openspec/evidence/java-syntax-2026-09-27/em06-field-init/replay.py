#!/usr/bin/env python3
"""Replay the EM-06 static-order and instance non-hoist Java 8 slice."""

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
INPUT = HERE / "input" / "em06"
JADX_REV = "2fb1b16386941660fda07e9017285aec40fcb37f"
PINS = {
    "test/java/jadx/tests/integration/others/TestFieldInit3.java": "9b67a2a4bed2ae8b17d6416306dde9f5ef407399b841dcf5796edfb9bc4ec0cf",
    "test/java/jadx/tests/integration/others/TestFieldInitInTryCatch.java": "7de1be987595913b00f06896631260846c142540cd61466b88b7a704e3488b8e",
    "test/java/jadx/tests/integration/others/TestFieldInitNegative.java": "3c6cdc9b9f96c1c72b3debfac1bf2587f1b650397d5ffa38e79a66abce80a0df",
    "test/java/jadx/tests/integration/others/TestFieldInitOrderStatic.java": "96cf60ae5a82309aa74b980f95652ff01e43497120a3d6114dcd00c4299015ef",
    "test/java/jadx/tests/integration/arrays/TestArrayInitField.java": "245af9a71da469252343fcc845300c7668d14383f6ef597ecb686177bb2d732c",
    "main/java/jadx/core/dex/visitors/ExtractFieldInit.java": "27127441f0667fd8ef3a3e4ac27031fed9c55054f3fb5797d869a64b97da8e68",
    "main/java/jadx/core/codegen/ClassGen.java": "9181b93a84d38b0cca68170e3886291da27ae5e5fcaf5535e4835b51ec3c5ba3",
}
EXPECTED = "a:ab:abc:abc\nsb2"


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def run(command, log, *, compact=False):
    result = subprocess.run([str(part) for part in command], capture_output=True, text=True, timeout=180)
    log.parent.mkdir(parents=True, exist_ok=True)
    if compact:
        log.write_text(
            f"exit={result.returncode}\nstdout_sha256={hashlib.sha256(result.stdout.encode()).hexdigest()}\n"
            f"stderr_sha256={hashlib.sha256(result.stderr.encode()).hexdigest()}\n"
        )
    else:
        log.write_text("\n".join(line.rstrip() for line in
                                 (result.stdout + result.stderr).splitlines()) + "\n")
    if result.returncode:
        raise RuntimeError(f"command failed ({result.returncode}): {log}")
    return result.stdout


def compile_run(name, source, output, temporary):
    classes = temporary / f"{name}-classes"
    classes.mkdir()
    run(["javac", "--release", "8", "-g:none", "-d", classes,
         source, INPUT / "Runner.java"], output / name / "javac.log")
    stdout = run(["java", "-Xverify:all", "-cp", classes, "em06.Runner"],
                 output / name / "runtime.log").strip()
    if stdout != EXPECTED:
        raise RuntimeError(f"{name} changed runtime output: {stdout!r}")
    return {"javac_exit": 0, "runtime_exit": 0, "stdout": stdout,
            "source_sha256": digest(source)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jarde", type=Path, required=True)
    parser.add_argument("--jadx", type=Path, required=True)
    parser.add_argument("--jadx-checkout", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    output = args.out.resolve()
    if output.exists() and any(output.iterdir()):
        parser.error("--out must be empty")
    output.mkdir(parents=True, exist_ok=True)
    revision = run(["git", "-C", args.jadx_checkout, "rev-parse", "HEAD"],
                   output / "jadx-revision.txt").strip()
    if revision != JADX_REV:
        raise RuntimeError("JADX checkout changed")
    for name, sha in PINS.items():
        if digest(args.jadx_checkout / "jadx-core/src" / name) != sha:
            raise RuntimeError(f"JADX pinned source changed: {name}")
    with tempfile.TemporaryDirectory(prefix="jarde-em06-") as temp_name:
        temporary = Path(temp_name)
        source = INPUT / "FieldOrder.java"
        original = compile_run("original", source, output, temporary)
        original_class_sha256 = digest(temporary / "original-classes/em06/FieldOrder.class")
        jar = temporary / "fixture.jar"
        run(["jar", "cf", jar, "-C", temporary / "original-classes", "."], output / "jar.log")
        jadx_dir = temporary / "jadx"
        run([args.jadx, "-d", jadx_dir, jar], output / "jadx.log")
        jadx_source = output / "source" / "jadx" / "em06" / "FieldOrder.java"
        jadx_source.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(jadx_dir / "sources/em06/FieldOrder.java", jadx_source)
        jarde_source = output / "source" / "jarde" / "em06" / "FieldOrder.java"
        jarde_source.parent.mkdir(parents=True, exist_ok=True)
        jarde_source.write_text(run(
            [args.jarde, "class-source", "--input", jar, "--class", "em06.FieldOrder",
             "--policy", "plain-jar", "--release", "8", "--evidence", "essential",
             "--format", "text"], output / "jarde.log", compact=True))
        jadx = compile_run("jadx", jadx_source, output, temporary)
        jarde = compile_run("jarde", jarde_source, output, temporary)
    jadx_text = jadx_source.read_text()
    jarde_text = jarde_source.read_text()
    for label, text in (("jadx", jadx_text), ("jarde", jarde_text)):
        if "int field = initField()" in text:
            raise RuntimeError(f"{label} moved a state-dependent constructor call to a field declaration")
        if "field = " not in text or "initField()" not in text:
            raise RuntimeError(f"{label} lost the instance assignment")
    result = {
        "jadx_revision": revision, "jadx_pins": PINS,
        "jarde_cli_sha256": digest(args.jarde),
        "original_class_sha256": original_class_sha256,
        "input_sha256": {p.name: digest(p) for p in INPUT.glob("*.java")},
        "original": original, "jadx": jadx, "jarde": jarde,
        "static_fields_inline": {
            "jadx": "String result =" in jadx_text,
            "jarde": "String result =" in jarde_text,
        },
        "instance_write_not_hoisted": True,
    }
    (output / "summary.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
