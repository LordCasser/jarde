#!/usr/bin/env python3
"""Freeze the Java 8 local-double anonymous capture comparison."""

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
    log.parent.mkdir(parents=True, exist_ok=True)
    result = subprocess.run([str(value) for value in args], text=True,
                            capture_output=True, timeout=120)
    output = result.stdout + result.stderr
    log.write_text("\n".join(line.rstrip() for line in output.splitlines())
                   + ("\n" if output.endswith("\n") else ""))
    return result


def compile_source(name, sources, work, out):
    classes = work / "classes" / name
    classes.mkdir(parents=True)
    result = run(["javac", "--release", "8", "-g:none", "-d", classes, *sources],
                 out / name / "javac.log")
    return result, classes


def verified_run(name, classes, out):
    result = run(["java", "-Xverify:all", "-cp", classes, "p.Runner"],
                 out / name / "runtime.log")
    if result.returncode or result.stdout.strip() != EXPECTED:
        raise RuntimeError(f"{name} runtime mismatch")
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
    with tempfile.TemporaryDirectory(prefix="jarde-dt08-") as temporary:
        work = Path(temporary)
        sources = sorted(INPUT.glob("*.java"))
        original_compile, original_classes = compile_source("original", sources, work, out)
        if original_compile.returncode:
            raise RuntimeError("original Java 8 input did not compile")
        original_run = verified_run("original", original_classes, out)
        jar = work / "input.jar"
        if run(["jar", "cf", jar, "-C", original_classes, "."], out / "jar.log").returncode:
            raise RuntimeError("jar failed")
        run(["javap", "-classpath", original_classes, "-c", "-p", "p.Capture$1"],
            out / "child-javap.log")

        jadx_dir = work / "jadx"
        if run([args.jadx, "-d", jadx_dir, jar], out / "jadx.log").returncode:
            raise RuntimeError("JADX failed")
        jadx_out = out / "jadx"
        jadx_out.mkdir()
        jadx_subject = jadx_dir / "sources" / "p" / "Capture.java"
        (jadx_out / "Capture.java").write_bytes(jadx_subject.read_bytes())
        jadx_compile, jadx_classes = compile_source(
            "jadx", [jadx_out / "Capture.java", INPUT / "Runner.java"], work, out)
        if jadx_compile.returncode:
            raise RuntimeError("JADX complete source did not compile")
        jadx_run = verified_run("jadx", jadx_classes, out)

        jarde_out = out / "jarde"
        jarde_out.mkdir()
        jarde_exits = {}
        for name in ("p.Capture", "p.Capture$1"):
            simple = name.split(".")[-1]
            result = run([args.jarde, "class-source", "--input", jar, "--class", name,
                          "--policy", "plain-jar", "--release", "8", "--evidence",
                          "essential", "--format", "text"],
                         jarde_out / (simple + ".diagnostics.log"))
            jarde_exits[name] = result.returncode
            (jarde_out / (simple + ".java")).write_text(result.stdout)
        report = run([args.jarde, "class-source", "--input", jar, "--class", "p.Capture",
                      "--policy", "plain-jar", "--release", "8", "--evidence",
                      "essential", "--format", "json"],
                     jarde_out / "Capture.json-diagnostics.log")
        refusals = [item["code"] for item in json.loads(report.stdout)["diagnostics"]]
        jarde_compile, _ = compile_source(
            "jarde", [*sorted(jarde_out.glob("*.java")), INPUT / "Runner.java"], work, out)
        jarde_source = (jarde_out / "Capture.java").read_text()
        child_source = (jarde_out / "Capture$1.java").read_text()
        if "anonymous_child_shape_unproved" not in refusals:
            raise RuntimeError("Jarde capture refusal changed")
        if "return new Runnable() {" not in jadx_subject.read_text():
            raise RuntimeError("JADX no longer recovers source-level capture")
        if "return new p.Capture$1(arg0);" not in jarde_source:
            raise RuntimeError("Jarde no longer has the frozen physical child reference")
        if "this.val$d = arg1;\n        super();" not in child_source:
            raise RuntimeError("Jarde physical child constructor shape changed")
        if jarde_compile.returncode == 0:
            raise RuntimeError("the frozen Jarde complete-source compiler gap disappeared")
        summary = {
            "fixed_jadx_revision": "2fb1b16386941660fda07e9017285aec40fcb37f",
            "jarde_cli_sha256": sha(args.jarde.resolve()),
            "input_sha256": {p.name: sha(p) for p in sources},
            "source_sha256": {
                "jadx": sha(jadx_out / "Capture.java"),
                "jarde": {p.name: sha(p) for p in sorted(jarde_out.glob("*.java"))},
            },
            "original": {"javac_exit": original_compile.returncode, "runtime": original_run},
            "jadx": {"javac_exit": jadx_compile.returncode, "runtime": jadx_run},
            "jarde": {"class_source_exits": jarde_exits,
                       "javac_exit": jarde_compile.returncode, "refusals": refusals},
        }
        (out / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
        print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
