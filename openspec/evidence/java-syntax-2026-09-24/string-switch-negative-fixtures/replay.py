#!/usr/bin/env python3
"""Rebuild the Java 8 negative String-switch cases and replay both decompilers."""
from __future__ import annotations

import hashlib
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
CLI = os.environ.get("JARDE_CLI")
if not CLI:
    raise SystemExit("set JARDE_CLI to the jarde-cli binary to test")
CLI = str(Path(CLI).resolve())
CASES = ("ExtraHashUse", "WrongHashBucket")
EXPECTED_CLASS_SHA = {
    "ExtraHashUse": "076e908fcc980f54a6aa09d49a1b63ebaeb6f9306431a0f8b9fad829daa910c5",
    "WrongHashBucket": "f0a22997ff6974a252ab5279b2aa1f07cb315ea2cde0b5e4dccdf443cd22dc16",
}
EXPECTED_RUNNER_SHA = {
    "ExtraHashUse": "fb1e514c75a7fb4b7c85caac2e30fb95a8fff90cfef09a8356538e2170f7ffec",
    "WrongHashBucket": "cce8cc510e30359cecd7226bb8de97ff0b6b0ef3617cb10e75475de87f653714",
}


def run(command: list[str], *, cwd: Path | None = None, check: bool = True,
        stdout: Path | None = None, stderr: Path | None = None) -> subprocess.CompletedProcess[str]:
    out = stdout.open("w") if stdout else subprocess.PIPE
    err = stderr.open("w") if stderr else subprocess.STDOUT
    try:
        result = subprocess.run(command, cwd=cwd, text=True, stdout=out, stderr=err)
    finally:
        if stdout:
            out.close()
        if stderr:
            err.close()
    if check and result.returncode:
        raise RuntimeError(f"exit {result.returncode}: {' '.join(command)}")
    return result


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def compile_sources(output: Path, *sources: Path, check: bool = True) -> subprocess.CompletedProcess[str]:
    output.mkdir(parents=True, exist_ok=True)
    return run(["javac", "--release", "8", "-g:none", "-d", str(output),
                *(str(source) for source in sources)], check=check)


def execute(output: Path, runner: str, capture: Path) -> None:
    run(["java", "-Xverify:all", "-cp", str(output), runner], stdout=capture)


with tempfile.TemporaryDirectory(prefix="jarde-string-switch-negative-") as temp_name:
    temp = Path(temp_name)
    original = temp / "original"
    for name in CASES:
        runner_digest = sha(HERE / f"{name}Runner.java")
        if runner_digest != EXPECTED_RUNNER_SHA[name]:
            raise SystemExit(f"{name}Runner.java SHA changed: {runner_digest}")
    compile_sources(original, *(HERE / f"{name}.java" for name in CASES),
                    *(HERE / f"{name}Runner.java" for name in CASES))
    for name in CASES:
        class_file = original / f"{name}.class"
        digest = sha(class_file)
        if digest != EXPECTED_CLASS_SHA[name]:
            raise SystemExit(f"{name}.class SHA changed: {digest}")
        shutil.copy2(class_file, HERE / "classes" / class_file.name)
        run(["javap", "-v", "-c", "-p", str(class_file)], stdout=HERE / f"{name}.javap.txt")
        execute(original, f"{name}Runner", HERE / f"{name}.original-run.txt")

        jarde_stderr = temp / f"{name}.jarde.stderr.txt"
        jarde_source = run([CLI, "class-source", "--input", str(class_file), "--class", name,
                            "--policy", "single-class", "--release", "8", "--format", "text"],
                           stderr=jarde_stderr)
        (HERE / f"{name}.jarde.java").write_text(jarde_source.stdout)
        shutil.copy2(jarde_stderr, HERE / f"{name}.jarde.stderr.txt")
        jarde = temp / f"jarde-{name}"
        jarde.mkdir()
        jarde_class_source = HERE / f"{name}.jarde.java"
        replay_jarde_source = jarde / f"{name}.java"
        shutil.copy2(jarde_class_source, replay_jarde_source)
        compile_sources(jarde, replay_jarde_source, HERE / f"{name}Runner.java")
        execute(jarde, f"{name}Runner", HERE / f"{name}.jarde-run.txt")
        if (HERE / f"{name}.original-run.txt").read_bytes() != (HERE / f"{name}.jarde-run.txt").read_bytes():
            raise SystemExit(f"Jarde runtime mismatch: {name}")

        jadx_dir = temp / f"jadx-{name}"
        run(["jadx", "-d", str(jadx_dir), str(class_file)], stdout=HERE / f"{name}.jadx.log")
        jadx_source = jadx_dir / "sources" / "defpackage" / f"{name}.java"
        source_text = jadx_source.read_text()
        # JADX inserts defpackage for a class that had no package in its class file.
        normalized = source_text.replace("package defpackage;\n\n", "", 1)
        (HERE / f"{name}.jadx.java").write_text(normalized)
        jadx = temp / f"jadx-compiled-{name}"
        jadx.mkdir()
        replay_jadx_source = jadx / f"{name}.java"
        shutil.copy2(HERE / f"{name}.jadx.java", replay_jadx_source)
        compile_result = compile_sources(jadx, replay_jadx_source,
                                         HERE / f"{name}Runner.java", check=False)
        (HERE / f"{name}.jadx-javac.log").write_text(
            compile_result.stdout or f"exit {compile_result.returncode}\n")
        if name == "ExtraHashUse":
            if compile_result.returncode == 0:
                raise SystemExit("expected JADX's ExtraHashUse recovery to fail compilation")
            if "r0" not in (HERE / f"{name}.jadx-javac.log").read_text():
                raise SystemExit("JADX ExtraHashUse failed for an unexpected reason")
        else:
            if compile_result.returncode:
                raise SystemExit("JADX WrongHashBucket should compile")
            execute(jadx, f"{name}Runner", HERE / f"{name}.jadx-run.txt")
            if (HERE / f"{name}.original-run.txt").read_bytes() != (HERE / f"{name}.jadx-run.txt").read_bytes():
                raise SystemExit("JADX runtime mismatch: WrongHashBucket")

print("PASS: javac --release 8 class hashes are frozen; original and Jarde execute equally under -Xverify:all.")
print("PASS: JADX preserves WrongHashBucket behavior; ExtraHashUse emits an unresolved r0 reference and does not compile.")
