#!/usr/bin/env python3
"""Compare Java 8 body-bearing generic throws against local JADX and Jarde."""

from __future__ import annotations

import hashlib
import os
from pathlib import Path
import subprocess
from tempfile import TemporaryDirectory


ROOT = Path(__file__).resolve().parents[4]
FIXTURE = Path(__file__).resolve().parent / "fixture"


def run(*args: object, env: dict[str, str] | None = None) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(
        [str(item) for item in args], cwd=ROOT, env=env, text=True, capture_output=True
    )
    if result.returncode and str(args[0]) != "javac":
        raise RuntimeError(f"{args}: {result.stdout}\n{result.stderr}")
    return result


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


with TemporaryDirectory(prefix="jarde-body-generic-throws-") as tmp:
    work = Path(tmp)
    env = os.environ.copy()
    env["CARGO_TARGET_DIR"] = str(work / "cargo-target")
    for variant, debug in (("g", "-g"), ("none", "-g:none")):
        original = work / f"original-{variant}"
        original.mkdir()
        built = run(
            "javac", "--release", "8", "-Xlint:-options", debug, "-d", original,
            FIXTURE / "BodyThrows.java", FIXTURE / "BodyThrowsCaller.java",
        )
        assert built.returncode == 0, built.stderr
        original_run = run("java", "-Xverify:all", "-cp", original, "bodythrows.BodyThrowsCaller")
        assert original_run.stdout.strip() == "throws=E"
        print(f"original {variant}: {original_run.stdout.strip()}")
        boundary = original / "bodythrows/BodyThrows.class"
        javap = run("javap", "-v", boundary).stdout
        assert "Exceptions:" in javap
        assert "java/lang/Exception" in javap
        assert "()V^TE;" in javap
        print("\n".join(line.strip() for line in javap.splitlines()
                        if "Signature:" in line or "Exceptions:" in line or "void run()" in line))
        jadx_out = work / f"jadx-{variant}"
        run("jadx", "-d", jadx_out, boundary)
        jadx_source = jadx_out / "sources/bodythrows/BodyThrows.java"
        jarde_out = work / f"jarde-{variant}"
        jarde_out.mkdir()
        jarde_source = jarde_out / "BodyThrows.java"
        jarde_source.write_text(run(
            "cargo", "run", "-q", "-p", "jarde-cli", "--", "class-source",
            "--input", boundary, "--class", "bodythrows.BodyThrows",
            "--policy", "single-class", env=env,
        ).stdout)
        for engine, source in (("jadx", jadx_source), ("jarde", jarde_source)):
            source_text = source.read_text()
            run_lines = [line.strip() for line in source_text.splitlines()
                         if "run()" in line or "Signature projection refused" in line]
            print(f"{engine} {variant}: " + " | ".join(run_lines))
            if engine == "jadx":
                assert "public void run() throws Exception" in source_text
            else:
                assert "public void run() throws E" in source_text
                assert "generic Signature projection refused for `run()V`" not in source_text
            output = work / f"{engine}-{variant}-compiled"
            output.mkdir()
            rebuilt = run(
                "javac", "--release", "8", "-Xlint:-options", "-d", output,
                source, FIXTURE / "BodyThrowsReflect.java",
            )
            assert rebuilt.returncode == 0, rebuilt.stderr
            reflected = run("java", "-Xverify:all", "-cp", output,
                            "bodythrows.BodyThrowsReflect").stdout.strip()
            expected_reflection = "throws=java.lang.Exception" if engine == "jadx" else "throws=E"
            assert reflected == expected_reflection, (engine, variant, reflected)
            print(f"{engine} {variant} reflection: {reflected}")
            caller = run(
                "javac", "--release", "8", "-Xlint:-options", "-cp", output,
                "-d", output, FIXTURE / "BodyThrowsCaller.java",
            )
            if engine == "jadx":
                assert caller.returncode != 0, "JADX's physical throws clause unexpectedly type-checks"
                assert "unreported exception" in caller.stderr or "必须对其进行捕获" in caller.stderr
                print(f"{engine} {variant} typed caller javac: expected rejection")
            else:
                assert caller.returncode == 0, caller.stderr
                caller_run = run("java", "-Xverify:all", "-cp", output,
                                 "bodythrows.BodyThrowsCaller").stdout.strip()
                assert caller_run == "throws=E", caller_run
                print(f"{engine} {variant} typed caller: {caller_run}")
            print(f"{engine} {variant} source sha256={sha(source)}")
        print(f"original {variant} class sha256={sha(boundary)}")
    for source in sorted(FIXTURE.glob("*.java")):
        print(f"fixture {source.name} sha256={sha(source)}")
    print(run("java", "-version").stderr.strip())
    print("jadx " + run("jadx", "--version").stdout.strip())
    print(run("cargo", "--version").stdout.strip())
