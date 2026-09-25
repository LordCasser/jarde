#!/usr/bin/env python3
"""Compare a Java 8 method-owned generic throws variable with a real body."""

from __future__ import annotations

import hashlib
import os
from pathlib import Path
import subprocess
from tempfile import TemporaryDirectory


ROOT = Path(__file__).resolve().parents[4]
FIXTURE = Path(__file__).resolve().parent / "fixture"


def run(*args: object, env: dict[str, str] | None = None) -> subprocess.CompletedProcess[str]:
    command = [str(item) for item in args]
    result = subprocess.run(command, cwd=ROOT, env=env, text=True, capture_output=True)
    if result.returncode and command[0] not in ("javac",):
        raise RuntimeError(f"{command}: {result.stdout}\n{result.stderr}")
    return result


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


with TemporaryDirectory(prefix="jarde-body-method-throws-") as tmp:
    work = Path(tmp)
    env = os.environ.copy()
    env["CARGO_TARGET_DIR"] = str(work / "cargo-target")
    for variant, debug in (("g", "-g"), ("none", "-g:none")):
        original = work / f"original-{variant}"
        original.mkdir()
        built = run(
            "javac", "--release", "8", "-Xlint:-options", debug, "-d", original,
            FIXTURE / "MethodBodyThrows.java", FIXTURE / "MethodBodyThrowsCaller.java",
            FIXTURE / "MethodBodyThrowsReflect.java",
        )
        assert built.returncode == 0, built.stderr
        output = run("java", "-Xverify:all", "-cp", original,
                     "methodbodythrows.MethodBodyThrowsCaller").stdout.strip()
        assert output == "called", output
        print(f"original {variant}: {output}")
        original_reflection = run("java", "-Xverify:all", "-cp", original,
                                  "methodbodythrows.MethodBodyThrowsReflect").stdout.strip()
        assert original_reflection == "parameters=1\nthrows=X", original_reflection
        print(f"original {variant} reflection: {original_reflection}")
        boundary = original / "methodbodythrows/MethodBodyThrows.class"
        javap = run("javap", "-v", boundary).stdout
        assert "public <X extends java.lang.Exception> void run() throws X;" in javap
        print("\n".join(line.strip() for line in javap.splitlines()
                        if "Signature:" in line or "Exceptions:" in line or "void run()" in line))
        jadx_out = work / f"jadx-{variant}"
        run("jadx", "-d", jadx_out, boundary)
        jadx_source = jadx_out / "sources/methodbodythrows/MethodBodyThrows.java"
        jarde_source = work / f"jarde-{variant}" / "MethodBodyThrows.java"
        jarde_source.parent.mkdir()
        jarde_source.write_text(run(
            "cargo", "run", "-q", "-p", "jarde-cli", "--", "class-source",
            "--input", boundary, "--class", "methodbodythrows.MethodBodyThrows",
            "--policy", "single-class", env=env,
        ).stdout)
        for engine, source in (("jadx", jadx_source), ("jarde", jarde_source)):
            source_text = source.read_text()
            if engine == "jadx":
                assert "public <X extends Exception> void run() throws Exception" in source_text
            else:
                assert "public <X extends java.lang.Exception> void run() throws X" in source_text
            print(f"{engine} {variant}: " + " | ".join(
                line.strip() for line in source_text.splitlines()
                if "run()" in line or "Signature projection refused" in line
            ))
            rebuilt = work / f"{engine}-{variant}-compiled"
            rebuilt.mkdir()
            class_build = run(
                "javac", "--release", "8", "-Xlint:-options", "-d", rebuilt,
                source, FIXTURE / "MethodBodyThrowsReflect.java",
            )
            print(f"{engine} {variant} class javac={class_build.returncode}")
            assert class_build.returncode == 0, class_build.stderr
            reflected = run("java", "-Xverify:all", "-cp", rebuilt,
                            "methodbodythrows.MethodBodyThrowsReflect").stdout.strip()
            expected = ("parameters=1\nthrows=java.lang.Exception" if engine == "jadx"
                        else "parameters=1\nthrows=X")
            assert reflected == expected, (engine, variant, reflected)
            print(reflected)
            caller = run(
                "javac", "--release", "8", "-Xlint:-options", "-cp", rebuilt,
                "-d", rebuilt, FIXTURE / "MethodBodyThrowsCaller.java",
            )
            if engine == "jadx":
                assert caller.returncode != 0, (engine, variant, caller.stdout, caller.stderr)
                print(f"{engine} {variant} typed caller javac={caller.returncode}")
                print(caller.stderr.strip())
            else:
                assert caller.returncode == 0, (engine, variant, caller.stdout, caller.stderr)
                executed = run("java", "-Xverify:all", "-cp", rebuilt,
                               "methodbodythrows.MethodBodyThrowsCaller")
                assert executed.stdout.strip() == "called", executed.stdout
                print(f"{engine} {variant} typed caller: {executed.stdout.strip()}")
            print(f"{engine} {variant} source sha256={sha(source)}")
        print(f"original {variant} class sha256={sha(boundary)}")
    for source in sorted(FIXTURE.glob("*.java")):
        print(f"fixture {source.name} sha256={sha(source)}")
    print(run("java", "-version").stderr.strip())
    print("jadx " + run("jadx", "--version").stdout.strip())
