#!/usr/bin/env python3
"""Replay the fixed Java 8 anonymous-initializer source comparison."""

import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile


HERE = Path(__file__).resolve().parent
INPUT = HERE / "input" / "p"
EXPECTED = "1\n8"
CLASSES = ("p.Base", "p.Subject", "p.Subject$1")


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run(command, log):
    result = subprocess.run(
        [str(value) for value in command], text=True, capture_output=True, timeout=120
    )
    log.write_text(result.stdout + result.stderr)
    return result


def compile_and_run(label, sources, out, runner):
    classes = out / label / "classes"
    classes.mkdir(parents=True)
    compile_result = run(
        ["javac", "--release", "8", "-g:none", "-d", classes, *sources],
        out / label / "javac.log",
    )
    if compile_result.returncode:
        raise RuntimeError(f"{label} complete source did not compile")
    result = run(
        ["java", "-Xverify:all", "-cp", classes, runner],
        out / label / "runtime.log",
    )
    if result.returncode or result.stdout.strip() != EXPECTED:
        raise RuntimeError(f"{label} did not match the original behavior")
    return {"javac_exit": compile_result.returncode, "runtime_exit": result.returncode,
            "runtime_stdout": result.stdout.strip()}


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
    with tempfile.TemporaryDirectory(prefix="jarde-dt09-") as temp_name:
        temp = Path(temp_name)
        original_sources = sorted(INPUT.glob("*.java"))
        original = compile_and_run("original", original_sources, temp, "p.Runner")
        jar = temp / "original.jar"
        if run(["jar", "cf", jar, "-C", temp / "original" / "classes", "."],
               out / "jar.log").returncode:
            raise RuntimeError("jar failed")

        jadx_root = temp / "jadx"
        if run([args.jadx, "-d", jadx_root, jar], out / "jadx.log").returncode:
            raise RuntimeError("JADX failed")
        jadx_sources = jadx_root / "sources" / "p"
        jadx_subject = (jadx_sources / "Subject.java").read_text()
        jadx_out = out / "jadx"
        jadx_out.mkdir()
        for name in ("Base.java", "Subject.java"):
            (jadx_out / name).write_bytes((jadx_sources / name).read_bytes())
        jadx = compile_and_run(
            "jadx", [jadx_out / "Base.java", jadx_out / "Subject.java", INPUT / "Runner.java"],
            out, "p.Runner",
        )

        jarde_out = out / "jarde"
        jarde_out.mkdir()
        source_exits = {}
        for class_name in CLASSES:
            result = run(
                [args.jarde, "class-source", "--input", jar, "--class", class_name,
                 "--policy", "plain-jar", "--release", "8", "--evidence", "essential",
                 "--format", "text"],
                jarde_out / (class_name.rsplit(".", 1)[1] + ".diagnostics.log"),
            )
            source_exits[class_name] = result.returncode
            (jarde_out / (class_name.rsplit(".", 1)[1] + ".java")).write_text(result.stdout)
        report_result = run(
            [args.jarde, "class-source", "--input", jar, "--class", "p.Subject",
             "--policy", "plain-jar", "--release", "8", "--evidence", "essential",
             "--format", "json"],
            jarde_out / "Subject.json-diagnostics.log",
        )
        report = json.loads(report_result.stdout)
        refusals = [entry["code"] for entry in report["diagnostics"]]
        jarde = compile_and_run(
            "jarde", [*sorted(jarde_out.glob("*.java")), INPUT / "Runner.java"],
            out, "p.Runner",
        )
        jarde_subject = (jarde_out / "Subject.java").read_text()
        if "anonymous_child_constructor_ast_missing" not in refusals:
            raise RuntimeError("the expected Jarde constructor proof refusal changed")
        if "new Base() {" not in jadx_subject or "new p.Subject$1()" not in jarde_subject:
            raise RuntimeError("the source syntax comparison changed")
        summary = {
            "fixed_jadx_revision": "2fb1b16386941660fda07e9017285aec40fcb37f",
            "jarde_cli_sha256": digest(args.jarde.resolve()),
            "input_sha256": {p.name: digest(p) for p in original_sources},
            "source_sha256": {
                "jadx": {p.name: digest(p) for p in sorted(jadx_out.glob("*.java"))},
                "jarde": {p.name: digest(p) for p in sorted(jarde_out.glob("*.java"))},
            },
            "original": original,
            "jadx": jadx,
            "jarde": jarde,
            "jarde_class_source_exits": source_exits,
            "jarde_refusals": refusals,
        }
        (out / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
        print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
