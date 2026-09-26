#!/usr/bin/env python3
"""Rebuild the fixed Jarde static-member source evidence without touching the baseline."""
from pathlib import Path
import hashlib
import json
import os
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[4]
FIX = ROOT / "tests/fixtures/proved-java-structure/static-member-basic"
EXPECTED = FIX / "class-sha256.txt"
CLASSES = ("StaticMemberBasic", "StaticMemberBasic$Leaf", "Named$Top")


def run(args, *, cwd=None, env=None):
    result = subprocess.run(args, cwd=cwd, env=env, text=True, capture_output=True)
    if result.returncode:
        raise RuntimeError(
            f"command failed ({result.returncode}): {args}\n{result.stdout}{result.stderr}"
        )
    return result


def save_result(path, result):
    path.write_text(result.stdout + result.stderr)


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    if len(sys.argv) != 2:
        raise SystemExit("usage: replay.py OUTPUT_DIRECTORY")
    output = Path(sys.argv[1]).resolve()
    output.mkdir(parents=True, exist_ok=False)
    source_dir = output / "jarde-source"
    source_dir.mkdir()

    with tempfile.TemporaryDirectory(prefix="jarde-dt02-cargo-") as cargo_target, tempfile.TemporaryDirectory(
        prefix="jarde-dt02-replay-"
    ) as temporary:
        work = Path(temporary)
        original = work / "original"
        original.mkdir()
        compile_original = run(
            [
                "javac",
                "--release",
                "8",
                "-g:none",
                "-d",
                str(original),
                str(FIX / "StaticMemberBasic.java"),
                str(FIX / "Named$Top.java"),
            ]
        )
        save_result(output / "original-javac.log", compile_original)
        for name in CLASSES:
            frozen = FIX / f"{name}.class"
            rebuilt = original / f"{name}.class"
            if digest(frozen) != digest(rebuilt):
                raise RuntimeError(f"recompiled baseline differs: {name}")
        (output / "class-sha256.txt").write_text(
            "".join(
                f"{digest(original / f'{name}.class')}  {name}.class\n"
                for name in sorted(CLASSES)
            )
        )
        for name in CLASSES:
            javap = run(["javap", "-p", "-c", "-s", "-classpath", str(original), name])
            save_result(output / f"javap-{name}.txt", javap)

        jar = work / "fixture.jar"
        packed = run(["jar", "--create", "--file", str(jar), "-C", str(original), "."])
        save_result(output / "jar.log", packed)
        env = os.environ.copy()
        env["CARGO_TARGET_DIR"] = cargo_target
        build = run(["cargo", "build", "-p", "jarde-cli", "--locked"], cwd=ROOT, env=env)
        save_result(output / "jarde-build.log", build)
        cli = Path(cargo_target) / "debug/jarde-cli"

        for name in CLASSES:
            report = run(
                [
                    str(cli),
                    "class-source",
                    "--input",
                    str(jar),
                    "--class",
                    name,
                    "--policy",
                    "plain-jar",
                    "--release",
                    "8",
                    "--format",
                    "json",
                ]
            )
            (output / f"report-{name}.json").write_text(report.stdout)
            text = run(
                [
                    str(cli),
                    "class-source",
                    "--input",
                    str(jar),
                    "--class",
                    name,
                    "--policy",
                    "plain-jar",
                    "--release",
                    "8",
                    "--format",
                    "text",
                ]
            )
            source_path = source_dir / f"{name}.java"
            source_path.write_text(text.stdout)

        root_source = (source_dir / "StaticMemberBasic.java").read_text()
        top_source = (source_dir / "Named$Top.java").read_text()
        child_source = (source_dir / "StaticMemberBasic$Leaf.java").read_text()
        for spelling in ("static Leaf make()", "return new Leaf();", "static class Leaf", "Leaf() {"):
            if spelling not in root_source:
                raise RuntimeError(f"fixed root source is missing {spelling!r}")
        if "StaticMemberBasic$Leaf()" in root_source:
            raise RuntimeError("fixed root source retained the binary constructor name")
        if "class Named$Top" not in top_source or "static class" in top_source:
            raise RuntimeError("dollar-name top-level control changed identity")
        if "class StaticMemberBasic$Leaf" not in child_source or "StaticMemberBasic$Leaf()" not in child_source:
            raise RuntimeError("independent physical child report lost its binary identity")

        fixed = work / "fixed"
        fixed.mkdir()
        javac = run(
            [
                "javac",
                "--release",
                "8",
                "-g:none",
                "-d",
                str(fixed),
                str(source_dir / "StaticMemberBasic.java"),
                str(source_dir / "Named$Top.java"),
            ]
        )
        save_result(output / "fixed-javac.log", javac)
        executed = run(["java", "-Xverify:all", "-cp", str(fixed), "StaticMemberBasic"])
        save_result(output / "fixed-run.log", executed)
        if executed.stdout != "9:4\n":
            raise RuntimeError(f"fixed runtime output differs: {executed.stdout!r}")

    print(json.dumps({"output": str(output), "runtime": "9:4"}, ensure_ascii=False))


if __name__ == "__main__":
    main()
