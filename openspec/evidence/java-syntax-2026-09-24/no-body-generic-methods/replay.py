#!/usr/bin/env python3
"""Repeat Java 8 original/JADX/Jarde no-body generic-method comparison."""

from __future__ import annotations

import hashlib
import os
from pathlib import Path
import subprocess
from tempfile import TemporaryDirectory


ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
FIXTURE = HERE / "fixture"


def run(*args: object, env: dict[str, str] | None = None) -> subprocess.CompletedProcess[str]:
    command = [str(arg) for arg in args]
    result = subprocess.run(command, cwd=ROOT, env=env, text=True, capture_output=True)
    if result.returncode and command[0] not in {"javac"}:
        raise RuntimeError(f"{command}: {result.stdout}\n{result.stderr}")
    return result


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


with TemporaryDirectory(prefix="jarde-no-body-generics-") as tmp:
    work = Path(tmp)
    env = os.environ.copy()
    env["CARGO_TARGET_DIR"] = str(work / "cargo-target")
    for variant, debug in (("g", "-g"), ("none", "-g:none")):
        original = work / f"original-{variant}"
        original.mkdir()
        boundary = original / "nobodygeneric/NoBodyGenericIdentity.class"
        built = run(
            "javac", "--release", "8", "-Xlint:-options", debug, "-d", original,
            FIXTURE / "NoBodyGenericIdentity.java", FIXTURE / "NoBodyGenericCaller.java",
        )
        assert built.returncode == 0, built.stderr
        executed = run("java", "-Xverify:all", "-cp", original, "nobodygeneric.NoBodyGenericCaller")
        print(f"original {variant}: {executed.stdout.strip()}")
        facts = run("javap", "-v", boundary).stdout
        print("\n".join(line.strip() for line in facts.splitlines()
                        if "Signature:" in line or "Exceptions:" in line or "abstract <" in line))
        jadx_out = work / f"jadx-{variant}"
        run("jadx", "-d", jadx_out, boundary)
        jadx_source = jadx_out / "sources/nobodygeneric/NoBodyGenericIdentity.java"
        jarde_out = work / f"jarde-{variant}"
        jarde_out.mkdir()
        jarde_source = jarde_out / "NoBodyGenericIdentity.java"
        jarde = run(
            "cargo", "run", "-q", "-p", "jarde-cli", "--", "class-source",
            "--input", boundary, "--class", "nobodygeneric.NoBodyGenericIdentity",
            "--policy", "single-class", env=env,
        )
        jarde_source.write_text(jarde.stdout)
        for engine, source in (("jadx", jadx_source), ("jarde", jarde_source)):
            print(f"{engine} {variant}: " + " | ".join(
                line.strip() for line in source.read_text().splitlines()
                if " echo(" in line or " checked(" in line
            ))
            output = work / f"{engine}-{variant}-compiled"
            output.mkdir()
            class_build = run(
                "javac", "--release", "8", "-Xlint:-options", "-d", output,
                source, FIXTURE / "NoBodyGenericReflect.java",
            )
            print(f"{engine} {variant} class javac={class_build.returncode}")
            assert class_build.returncode == 0, class_build.stderr
            reflect = run("java", "-Xverify:all", "-cp", output, "nobodygeneric.NoBodyGenericReflect")
            print(reflect.stdout.strip())
            caller = run(
                "javac", "--release", "8", "-Xlint:-options", "-cp", output,
                "-d", output, FIXTURE / "NoBodyGenericCaller.java",
            )
            print(f"{engine} {variant} caller javac={caller.returncode}")
            if caller.returncode:
                print(caller.stderr.strip())
            else:
                print(run("java", "-Xverify:all", "-cp", output,
                          "nobodygeneric.NoBodyGenericCaller").stdout.strip())
            print(f"{engine} {variant} source sha256={sha(source)}")
        print(f"original {variant} class sha256={sha(boundary)}")
    for variant, debug in (("g", "-g"), ("none", "-g:none")):
        original = work / f"interface-original-{variant}"
        original.mkdir()
        boundary = original / "nobodygeneric/RootGenericInterface.class"
        built = run(
            "javac", "--release", "8", "-Xlint:-options", debug, "-d", original,
            FIXTURE / "RootGenericInterface.java", FIXTURE / "RootGenericCaller.java",
        )
        assert built.returncode == 0, built.stderr
        executed = run("java", "-Xverify:all", "-cp", original, "nobodygeneric.RootGenericCaller")
        print(f"interface original {variant}: {executed.stdout.strip()}")
        facts = run("javap", "-v", boundary).stdout
        print("\n".join(line.strip() for line in facts.splitlines()
                        if "Signature:" in line or "Exceptions:" in line or "abstract <" in line))
        jadx_out = work / f"interface-jadx-{variant}"
        run("jadx", "-d", jadx_out, boundary)
        jadx_source = jadx_out / "sources/nobodygeneric/RootGenericInterface.java"
        jarde_out = work / f"interface-jarde-{variant}"
        jarde_out.mkdir()
        jarde_source = jarde_out / "RootGenericInterface.java"
        jarde = run(
            "cargo", "run", "-q", "-p", "jarde-cli", "--", "class-source",
            "--input", boundary, "--class", "nobodygeneric.RootGenericInterface",
            "--policy", "single-class", env=env,
        )
        jarde_source.write_text(jarde.stdout)
        for engine, source in (("jadx", jadx_source), ("jarde", jarde_source)):
            print(f"interface {engine} {variant}: " + " | ".join(
                line.strip() for line in source.read_text().splitlines()
                if " echo(" in line or " raise(" in line
            ))
            output = work / f"interface-{engine}-{variant}-compiled"
            output.mkdir()
            class_build = run(
                "javac", "--release", "8", "-Xlint:-options", "-d", output,
                source, FIXTURE / "RootGenericReflect.java",
            )
            print(f"interface {engine} {variant} class javac={class_build.returncode}")
            assert class_build.returncode == 0, class_build.stderr
            print(run("java", "-Xverify:all", "-cp", output,
                      "nobodygeneric.RootGenericReflect").stdout.strip())
            caller = run(
                "javac", "--release", "8", "-Xlint:-options", "-cp", output,
                "-d", output, FIXTURE / "RootGenericCaller.java",
            )
            print(f"interface {engine} {variant} caller javac={caller.returncode}")
            if caller.returncode:
                print(caller.stderr.strip())
            else:
                print(run("java", "-Xverify:all", "-cp", output,
                          "nobodygeneric.RootGenericCaller").stdout.strip())
            print(f"interface {engine} {variant} source sha256={sha(source)}")
        print(f"interface original {variant} class sha256={sha(boundary)}")
    for source in sorted(FIXTURE.glob("*.java")):
        print(f"fixture {source.name} sha256={sha(source)}")
    print(run("java", "-version").stderr.strip())
    print("jadx " + run("jadx", "--version").stdout.strip())
    print(run("cargo", "--version").stdout.strip())
