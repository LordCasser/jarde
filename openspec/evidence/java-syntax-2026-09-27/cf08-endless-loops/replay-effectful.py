#!/usr/bin/env python3
"""Replay the CF-08 effectful dual-exit fixture against pinned JADX and Jarde."""

import argparse
import json
from pathlib import Path
import shutil
import subprocess
import tempfile

from replay import JADX_REV, PINS, digest, run


HERE = Path(__file__).resolve().parent
INPUT = HERE / "input/effectful/cf08effects"
EXPECTED = "8:1\n3:0\n8:1"


def compile_run(label, source, runner, out, temp):
    classes = temp / f"{label}-classes"
    classes.mkdir()
    compiled = run(["javac", "--release", "8", "-g:none", "-d", classes,
                    source, runner], out / label / "javac.log")
    answer = {"source_sha256": digest(source), "javac_exit": compiled.returncode,
              "javac_stderr": compiled.stderr.strip()}
    if compiled.returncode:
        return answer, classes
    try:
        executed = subprocess.run(["java", "-Xverify:all", "-cp", str(classes),
                                   "cf08effects.Runner"], capture_output=True, text=True, timeout=5)
    except subprocess.TimeoutExpired:
        (out / label / "runtime.log").write_text("timeout=5s\n")
        answer["runtime_timeout"] = True
        return answer, classes
    (out / label / "runtime.log").write_text(
        f"exit={executed.returncode}\nstdout:\n{executed.stdout}\nstderr:\n{executed.stderr}")
    answer.update(runtime_exit=executed.returncode,
                  stdout=executed.stdout.strip(),
                  runtime_stderr=executed.stderr.strip())
    return answer, classes


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("jarde", "jadx", "jadx-checkout", "out"):
        parser.add_argument("--" + name, required=True, type=Path)
    parser.add_argument("--require-jarde", action="store_true",
                        help="require complete Jarde source and matching runtime")
    args = parser.parse_args()
    out = args.out.resolve()
    if out.exists() and any(out.iterdir()):
        parser.error("--out must be absent or empty")
    out.mkdir(parents=True, exist_ok=True)
    revision = run(["git", "-C", args.jadx_checkout, "rev-parse", "HEAD"],
                   out / "jadx-revision.log")
    if revision.returncode or revision.stdout.strip() != JADX_REV:
        raise RuntimeError("JADX checkout changed")
    for path, expected in PINS.items():
        if digest(args.jadx_checkout / path) != expected:
            raise RuntimeError(f"pinned JADX source changed: {path}")
    source = INPUT / "EffectfulExits.java"
    runner = INPUT / "Runner.java"
    with tempfile.TemporaryDirectory(prefix="jarde-cf08-effectful-") as temp_name:
        temp = Path(temp_name)
        original, classes = compile_run("original", source, runner, out, temp)
        if original.get("runtime_exit") or original.get("stdout") != EXPECTED:
            raise RuntimeError("original Java 8 fixture changed")
        class_file = classes / "cf08effects/EffectfulExits.class"
        class_digest = digest(class_file)
        if class_digest != "2e27cffb9361cbd78a689b404457b68d6bc31f1739362659ce71c0e276c0bae8":
            raise RuntimeError("frozen class digest changed")
        if run(["javap", "-classpath", classes, "-c", "-p", "cf08effects.EffectfulExits"],
               out / "javap.log").returncode:
            raise RuntimeError("javap failed")
        region_run = run([args.jarde, "recover", "--input", class_file,
                          "--class-name", "cf08effects.EffectfulExits", "--method-name", "pick",
                          "--descriptor", "([I)I", "--policy", "single-class",
                          "--release", "8", "--evidence", "region_details",
                          "--format", "json"], out / "region-report.log", compact=True)
        if region_run.returncode:
            raise RuntimeError("Jarde region evidence failed")
        recovery = json.loads(region_run.stdout)["recovered"]["recovery"]
        (out / "regions.json").write_text(json.dumps({
            "representation": recovery["representation"],
            "quality": recovery["quality"],
            "regions": recovery["regions"],
            "diagnostics": recovery["diagnostics"],
        }, indent=2) + "\n")
        jar = temp / "input.jar"
        if run(["jar", "cf", jar, "-C", classes, "."], out / "jar.log").returncode:
            raise RuntimeError("jar failed")
        jadx_root = temp / "jadx"
        if run([args.jadx, "-d", jadx_root, jar], out / "jadx.log").returncode:
            raise RuntimeError("JADX failed")
        jadx_source = out / "source/jadx/cf08effects/EffectfulExits.java"
        jadx_source.parent.mkdir(parents=True)
        shutil.copy2(jadx_root / "sources/cf08effects/EffectfulExits.java", jadx_source)
        recovered = run([args.jarde, "class-source", "--input", jar,
                         "--class", "cf08effects.EffectfulExits", "--policy", "plain-jar",
                         "--release", "8", "--format", "text"],
                        out / "jarde.log", compact=True)
        if recovered.returncode:
            raise RuntimeError("Jarde CLI failed")
        jarde_source = out / "source/jarde/cf08effects/EffectfulExits.java"
        jarde_source.parent.mkdir(parents=True)
        jarde_source.write_text(recovered.stdout)
        jadx, _ = compile_run("jadx", jadx_source, runner, out, temp)
        jarde, _ = compile_run("jarde", jarde_source, runner, out, temp)
        summary = {
            "jadx_revision": JADX_REV,
            "jadx_pins": PINS,
            "input_sha256": digest(source),
            "runner_sha256": digest(runner),
            "original_class_sha256": class_digest,
            "jarde_cli_sha256": digest(args.jarde),
            "original": original,
            "jadx": jadx,
            "jarde": jarde,
        }
        (out / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
        print(json.dumps(summary, indent=2))
        if jadx.get("javac_exit") or jadx.get("runtime_exit") or jadx.get("stdout") != EXPECTED:
            raise RuntimeError("complete JADX Java 8 replay differs")
        if args.require_jarde and (
            jarde.get("javac_exit") or jarde.get("runtime_exit")
            or jarde.get("stdout") != EXPECTED or "@bytecode" in jarde_source.read_text()
        ):
            raise RuntimeError("complete Jarde Java 8 replay differs")


if __name__ == "__main__":
    main()
