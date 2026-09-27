#!/usr/bin/env python3
"""Replay the fixed DT-08 comparison with the proved local-double projection enabled."""

import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile


HERE = Path(__file__).resolve().parent
INPUT = HERE / "input" / "p"
EXPECTED = "2.5\n-0.0"


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run(args, log):
    result = subprocess.run([str(value) for value in args], text=True,
                            capture_output=True, timeout=120)
    output = result.stdout + result.stderr
    log.parent.mkdir(parents=True, exist_ok=True)
    log.write_text("\n".join(line.rstrip() for line in output.splitlines())
                   + ("\n" if output.endswith("\n") else ""))
    return result


def compile_sources(name, sources, work, out):
    classes = work / "classes" / name
    classes.mkdir(parents=True)
    result = run(["javac", "--release", "8", "-g:none", "-d", classes, *sources],
                 out / name / "javac.log")
    if result.returncode:
        raise RuntimeError(f"{name} Java 8 compilation failed")
    return classes


def verified_run(name, classes, out):
    result = run(["java", "-Xverify:all", "-cp", classes, "p.Runner"],
                 out / name / "runtime.log")
    if result.returncode or result.stdout.strip() != EXPECTED:
        raise RuntimeError(f"{name} verified runtime mismatch")
    return {"exit": result.returncode, "stdout": result.stdout.strip()}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jarde", type=Path, required=True)
    parser.add_argument("--jadx", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    out = args.out.resolve()
    if out.exists() and any(out.iterdir()):
        parser.error("--out must be absent or empty")
    out.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="jarde-dt08-implementation-") as temporary:
        work = Path(temporary)
        inputs = sorted(INPUT.glob("*.java"))
        original_classes = compile_sources("original", inputs, work, out)
        original_run = verified_run("original", original_classes, out)
        jar = work / "input.jar"
        if run(["jar", "cf", jar, "-C", original_classes, "."], out / "jar.log").returncode:
            raise RuntimeError("jar failed")

        jadx_dir = work / "jadx"
        if run([args.jadx, "-d", jadx_dir, jar], out / "jadx.log").returncode:
            raise RuntimeError("JADX failed")
        jadx_out = out / "jadx"
        jadx_out.mkdir()
        jadx_source = jadx_dir / "sources" / "p" / "Capture.java"
        (jadx_out / "Capture.java").write_bytes(jadx_source.read_bytes())
        jadx_classes = compile_sources("jadx", [jadx_out / "Capture.java", INPUT / "Runner.java"], work, out)
        jadx_run = verified_run("jadx", jadx_classes, out)

        jarde_out = out / "jarde"
        jarde_out.mkdir()
        class_exits = {}
        for name in ("p.Capture", "p.Capture$1"):
            simple = name.split(".")[-1]
            result = run([args.jarde, "class-source", "--input", jar, "--class", name,
                          "--policy", "plain-jar", "--release", "8", "--evidence",
                          "essential", "--format", "text"],
                         jarde_out / (simple + ".diagnostics.log"))
            class_exits[name] = result.returncode
            (jarde_out / (simple + ".java")).write_text(result.stdout)
            if result.returncode:
                raise RuntimeError(f"Jarde could not present {name}")
        root_source = jarde_out / "Capture.java"
        child_source = jarde_out / "Capture$1.java"
        if "return new java.lang.Runnable() {" not in root_source.read_text():
            raise RuntimeError("Jarde did not emit the proved anonymous Runnable")
        if "Capture$1" in root_source.read_text() or "println(arg0)" not in root_source.read_text():
            raise RuntimeError("Jarde root projection retained the physical child or lost its capture")
        if "val$d" not in child_source.read_text():
            raise RuntimeError("the physical child is no longer independently queryable")
        jarde_classes = compile_sources("jarde-complete", [root_source, INPUT / "Runner.java"], work, out)
        jarde_run = verified_run("jarde-complete", jarde_classes, out)

        summary = {
            "fixed_jadx_revision": "2fb1b16386941660fda07e9017285aec40fcb37f",
            "jarde_cli_sha256": sha(args.jarde.resolve()),
            "input_sha256": {path.name: sha(path) for path in inputs},
            "source_sha256": {
                "jadx": sha(jadx_out / "Capture.java"),
                "jarde": {path.name: sha(path) for path in sorted(jarde_out.glob("*.java"))},
            },
            "original": {"javac_exit": 0, "runtime": original_run},
            "jadx": {"javac_exit": 0, "runtime": jadx_run},
            "jarde": {
                "class_source_exits": class_exits,
                "complete_source_javac_exit": 0,
                "runtime": jarde_run,
                "root_is_anonymous": True,
                "physical_child_queryable": True,
            },
            "frozen_jarde_baseline": "javac_exit=1; refusal=anonymous_child_shape_unproved",
        }
        (out / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
        print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
