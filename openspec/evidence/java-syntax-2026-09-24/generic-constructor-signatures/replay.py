#!/usr/bin/env python3
"""Java 8 generic-constructor source/JADX/Jarde recompile comparison."""

from __future__ import annotations

import hashlib
import os
from pathlib import Path
import subprocess
from tempfile import TemporaryDirectory


ROOT = Path(__file__).resolve().parents[4]
FIXTURE = Path(__file__).resolve().parent / "fixture"


def run(*args: object, env: dict[str, str] | None = None) -> subprocess.CompletedProcess[str]:
    command = [str(arg) for arg in args]
    result = subprocess.run(command, cwd=ROOT, env=env, text=True, capture_output=True)
    if result.returncode and command[0] != "javac":
        raise RuntimeError(f"{command}: {result.stdout}\n{result.stderr}")
    return result


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


with TemporaryDirectory(prefix="jarde-generic-constructor-") as tmp:
    work = Path(tmp)
    env = os.environ.copy()
    env["CARGO_TARGET_DIR"] = str(work / "cargo-target")
    for variant, debug in (("g", "-g"), ("none", "-g:none")):
        original = work / f"original-{variant}"
        original.mkdir()
        built = run(
            "javac", "--release", "8", "-Xlint:-options", debug, "-d", original,
            FIXTURE / "GenericConstructor.java", FIXTURE / "GenericConstructorCaller.java",
        )
        assert built.returncode == 0, built.stderr
        original_result = run(
            "java", "-Xverify:all", "-cp", original,
            "genericctor.GenericConstructorCaller",
        ).stdout.strip()
        assert "types=T" in original_result and "parameter=T" in original_result
        print(f"original {variant}: {original_result}")
        invalid_original = run(
            "javac", "--release", "8", "-Xlint:-options", "-cp", original,
            "-d", original, FIXTURE / "GenericConstructorTypeCheck.java",
        )
        assert invalid_original.returncode != 0
        print(f"original {variant} invalid caller javac={invalid_original.returncode}")
        boundary = original / "genericctor/GenericConstructor.class"
        javap = run("javap", "-v", boundary).stdout
        print("\n".join(line.strip() for line in javap.splitlines()
                        if "Signature:" in line or "<T extends" in line or "descriptor:" in line))
        jadx_out = work / f"jadx-{variant}"
        run("jadx", "-d", jadx_out, boundary)
        jadx_source = jadx_out / "sources/genericctor/GenericConstructor.java"
        jarde_out = work / f"jarde-{variant}"
        jarde_out.mkdir()
        jarde_source = jarde_out / "GenericConstructor.java"
        jarde = run(
            "cargo", "run", "-q", "-p", "jarde-cli", "--", "class-source",
            "--input", boundary, "--class", "genericctor.GenericConstructor",
            "--policy", "single-class", env=env,
        )
        jarde_source.write_text(jarde.stdout)
        for engine, source in (("jadx", jadx_source), ("jarde", jarde_source)):
            print(f"{engine} {variant}: " + " | ".join(
                line.strip() for line in source.read_text().splitlines()
                if "GenericConstructor(" in line
            ))
            if engine == "jarde" and variant == "g":
                print(source.read_text())
            output = work / f"{engine}-{variant}-compiled"
            output.mkdir()
            class_build = run(
                "javac", "--release", "8", "-Xlint:-options", "-d", output,
                source, FIXTURE / "GenericConstructorReflect.java",
            )
            assert class_build.returncode == 0, class_build.stderr
            print(f"{engine} {variant} class javac={class_build.returncode}")
            reflected = run("java", "-Xverify:all", "-cp", output,
                            "genericctor.GenericConstructorReflect").stdout.strip()
            assert reflected == "types=1\nparameter=T", (engine, variant, reflected)
            print(reflected)
            caller = run(
                "javac", "--release", "8", "-Xlint:-options", "-cp", output,
                "-d", output, FIXTURE / "GenericConstructorCaller.java",
            )
            assert caller.returncode == 0, caller.stderr
            print(f"{engine} {variant} caller javac={caller.returncode}")
            invalid = run(
                "javac", "--release", "8", "-Xlint:-options", "-cp", output,
                "-d", output, FIXTURE / "GenericConstructorTypeCheck.java",
            )
            assert invalid.returncode != 0, (engine, variant)
            print(f"{engine} {variant} invalid caller javac={invalid.returncode}")
            executed = run("java", "-Xverify:all", "-cp", output,
                           "genericctor.GenericConstructorCaller")
            assert "types=T" in executed.stdout and "parameter=T" in executed.stdout
            print(f"{engine} {variant} caller run={executed.returncode}")
            print(executed.stdout.strip())
            print(f"{engine} {variant} source sha256={sha(source)}")
        print(f"original {variant} class sha256={sha(boundary)}")
    for source in sorted(FIXTURE.glob("*.java")):
        print(f"fixture {source.name} sha256={sha(source)}")
    print(run("java", "-version").stderr.strip())
    print("jadx " + run("jadx", "--version").stdout.strip())
    print(run("cargo", "--version").stdout.strip())
