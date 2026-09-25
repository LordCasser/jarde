#!/usr/bin/env python3
"""Compare a direct-return generic method with a method-owned checked exception."""

from __future__ import annotations

import argparse
import hashlib
import os
from pathlib import Path
import subprocess
from tempfile import TemporaryDirectory


ROOT = Path(__file__).resolve().parents[4]
FIXTURE = Path(__file__).resolve().parent / "fixture/probe"
parser = argparse.ArgumentParser()
parser.add_argument("--mode", choices=("baseline", "recovered"), default="recovered")
args = parser.parse_args()


def run(*command: object, env: dict[str, str] | None = None) -> subprocess.CompletedProcess[str]:
    return subprocess.run([str(part) for part in command], cwd=ROOT, env=env,
                          capture_output=True, text=True)


def success(result: subprocess.CompletedProcess[str], label: str) -> str:
    assert result.returncode == 0, (label, result.stdout, result.stderr)
    return result.stdout


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


with TemporaryDirectory(prefix="jarde-static-local-throws-") as tmp:
    work = Path(tmp)
    binary_from_environment = os.environ.get("JARDE_CLI")
    if binary_from_environment:
        binary = Path(binary_from_environment).resolve()
        assert binary.is_file(), binary
    else:
        env = os.environ.copy()
        env["CARGO_TARGET_DIR"] = str(work / "cargo-target")
        success(run("cargo", "build", "-q", "-p", "jarde-cli", env=env), "build Jarde")
        binary = work / "cargo-target/debug/jarde-cli"
    for variant, debug in (("g", "-g"), ("none", "-g:none")):
        original = work / f"original-{variant}"
        original.mkdir()
        success(run("javac", "--release", "8", "-Xlint:-options", debug, "-d", original,
                    *sorted(FIXTURE.glob("*.java"))), f"compile original {variant}")
        original_output = success(run("java", "-Xverify:all", "-cp", original,
                                      "probe.Caller"), "run original").strip()
        assert original_output == "ok", original_output
        original_reflection = success(run("java", "-Xverify:all", "-cp", original,
                                          "probe.Reflect"), "reflect original").strip()
        assert original_reflection == "params=2\nreturn=T\nthrows=X", original_reflection
        class_file = original / "probe/StaticThrows.class"
        javap = success(run("javap", "-v", class_file), "javap")
        assert "(TT;)TT;^TX;" in javap and "throws java.lang.Exception" in javap
        print(f"original {variant}: caller={original_output}; reflection={original_reflection!r}; "
              f"class sha256={sha(class_file)}")
        jadx_dir = work / f"jadx-{variant}"
        success(run("jadx", "-d", jadx_dir, class_file), f"JADX {variant}")
        jadx_source = jadx_dir / "sources/probe/StaticThrows.java"
        jarde_result = success(run(binary, "class-source", "--input", class_file,
                                   "--class", "probe.StaticThrows", "--policy",
                                   "single-class"), f"Jarde {variant}")
        jarde_dir = work / f"jarde-{variant}/probe"
        jarde_dir.mkdir(parents=True)
        jarde_source = jarde_dir / "StaticThrows.java"
        assert jarde_result.startswith("// jarde:") and jarde_result.rstrip().endswith("}")
        jarde_source.write_text(jarde_result)
        for engine, source in (("jadx", jadx_source), ("jarde", jarde_source)):
            source_text = source.read_text()
            declaration = next(line.strip() for line in source_text.splitlines()
                               if " echo(" in line and "public " in line)
            if engine == "jadx":
                assert "<T, X extends Exception> T echo(T " in declaration, declaration
                assert "throws Exception" in declaration, declaration
                expected_reflection = "params=2\nreturn=T\nthrows=java.lang.Exception"
            elif args.mode == "baseline":
                assert "public static java.lang.Object echo(java.lang.Object " in declaration
                assert "throws java.lang.Exception" in declaration
                assert "generic_source_shape_unproved" in source_text
                expected_reflection = ("params=0\nreturn=java.lang.Object\n"
                                       "throws=java.lang.Exception")
            else:
                assert "<T extends java.lang.Object, X extends java.lang.Exception>" in declaration
                assert " T echo(T " in declaration and "throws X" in declaration
                expected_reflection = "params=2\nreturn=T\nthrows=X"
            rebuilt = work / f"rebuilt-{engine}-{variant}"
            rebuilt.mkdir()
            success(run("javac", "--release", "8", "-Xlint:-options", "-d", rebuilt,
                        source, FIXTURE / "Reflect.java"), f"compile {engine} {variant}")
            reflection = success(run("java", "-Xverify:all", "-cp", rebuilt,
                                     "probe.Reflect"), f"reflect {engine} {variant}").strip()
            assert reflection == expected_reflection, (engine, variant, reflection)
            caller = run("javac", "--release", "8", "-Xlint:-options", "-cp", rebuilt,
                         "-d", rebuilt, FIXTURE / "Caller.java")
            if engine == "jarde" and args.mode == "recovered":
                success(caller, f"typed caller {engine} {variant}")
                executed = success(run("java", "-Xverify:all", "-cp", rebuilt,
                                       "probe.Caller"), "run recovered caller").strip()
                assert executed == "ok", executed
            else:
                assert caller.returncode != 0, (engine, variant, caller.stdout, caller.stderr)
            print(f"{engine} {variant}: {declaration}; reflection={reflection!r}; "
                  f"caller javac={caller.returncode}; source sha256={sha(source)}")
    for source in sorted(FIXTURE.glob("*.java")):
        print(f"fixture {source.name} sha256={sha(source)}")
    print(success(run("jadx", "--version"), "JADX version").strip())
