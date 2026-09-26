#!/usr/bin/env python3
"""Replay boolean field-store recovery against original and patched Java 8 class files."""

from __future__ import annotations

import hashlib
import os
import shutil
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]
FIXTURE = ROOT / "tests/fixtures/p3-conditional-values/field-writes"
EVIDENCE = Path(__file__).resolve().parent


def run(args: list[str], *, cwd: Path | None = None, env: dict[str, str] | None = None):
    return subprocess.run(args, cwd=cwd, env=env, capture_output=True, text=True, timeout=120)


def compile_and_run(source: Path, classes: Path, *extra_sources: Path) -> str:
    classes.mkdir(parents=True, exist_ok=True)
    result = run(
        ["javac", "--release", "8", "-g:source,lines,vars", "-Xlint:-options", "-classpath", str(classes),
         "-d", str(classes), str(source),
         *(str(item) for item in extra_sources)]
    )
    if result.returncode:
        raise RuntimeError(f"javac failed for {source}:\n{result.stdout}{result.stderr}")
    execution = run(["java", "-Xverify:all", "-cp", str(classes), "Runner"])
    if execution.returncode:
        raise RuntimeError(f"java failed for {source}:\n{execution.stdout}{execution.stderr}")
    return execution.stdout


def replay_jarde(class_file: Path, directory: Path, target: Path) -> str:
    directory.mkdir()
    env = os.environ.copy()
    env["CARGO_TARGET_DIR"] = str(target)
    rendered = run(
        ["cargo", "run", "-q", "-p", "jarde-cli", "--", "class-source", "--evidence", "all",
         "--input", str(class_file), "--policy", "single-class", "--class", "ConditionalFieldWrites"],
        cwd=ROOT,
        env=env,
    )
    if rendered.returncode:
        raise RuntimeError(f"Jarde failed for {class_file}:\n{rendered.stdout}{rendered.stderr}")
    source = directory / "ConditionalFieldWrites.java"
    source.write_text(rendered.stdout)
    shutil.copy2(FIXTURE / "Runner.java", directory / "Runner.java")
    return compile_and_run(source, directory / "classes", directory / "Runner.java")


def main() -> None:
    EVIDENCE.mkdir(parents=True, exist_ok=True)
    class_files = (
        FIXTURE / "ConditionalFieldWrites.class",
        FIXTURE / "ConditionalFieldWritesNon01.class",
    )
    hashes = []
    for class_file, label in zip(class_files, ("original", "patched")):
        hashes.append(f"{label} {hashlib.sha256(class_file.read_bytes()).hexdigest()}\n")
        inspection = run(["javap", "-c", "-p", "-s", str(class_file)])
        if inspection.returncode:
            raise RuntimeError(f"javap failed for {class_file}:\n{inspection.stdout}{inspection.stderr}")
        (EVIDENCE / f"javap-{label}.txt").write_text(inspection.stdout)
    (EVIDENCE / "class-hashes.txt").write_text("".join(hashes))

    with tempfile.TemporaryDirectory(prefix="jarde-conditional-field-writes-") as temporary:
        work = Path(temporary)
        original_classes = work / "original-classes"
        original_output = compile_and_run(
            FIXTURE / "ConditionalFieldWrites.java", original_classes, FIXTURE / "Runner.java"
        )
        rebuilt = (original_classes / "ConditionalFieldWrites.class").read_bytes()
        frozen = (FIXTURE / "ConditionalFieldWrites.class").read_bytes()
        if rebuilt != frozen:
            raise AssertionError("javac rebuild of ConditionalFieldWrites.class changed bytes")
        patched_classes = work / "patched-classes"
        patched_classes.mkdir()
        shutil.copy2(FIXTURE / "ConditionalFieldWritesNon01.class",
                     patched_classes / "ConditionalFieldWrites.class")
        shutil.copy2(FIXTURE / "Runner.java", work / "patched-classes/Runner.java")
        patched_output = compile_and_run(
            patched_classes / "Runner.java", patched_classes
        )

        target = work / "cargo-target"
        jarde_original = replay_jarde(FIXTURE / "ConditionalFieldWrites.class",
                                      work / "jarde-original", target)
        jarde_patched = replay_jarde(FIXTURE / "ConditionalFieldWritesNon01.class",
                                     work / "jarde-patched", target)

        for label, actual, expected in (
            ("Jarde original", jarde_original, original_output),
            ("Jarde patched", jarde_patched, patched_output),
        ):
            if actual != expected:
                raise AssertionError(f"{label} differs:\nexpected={expected!r}\nactual={actual!r}")
        (EVIDENCE / "original-output.txt").write_text(original_output)
        (EVIDENCE / "patched-output.txt").write_text(patched_output)
        (EVIDENCE / "jarde-original-output.txt").write_text(jarde_original)
        (EVIDENCE / "jarde-patched-output.txt").write_text(jarde_patched)
        shutil.copy2(work / "jarde-original/ConditionalFieldWrites.java",
                     EVIDENCE / "jarde-original.java")
        shutil.copy2(work / "jarde-patched/ConditionalFieldWrites.java",
                     EVIDENCE / "jarde-patched.java")

        jadx_dir = work / "jadx-patched"
        jadx = run(["jadx", "-d", str(jadx_dir),
                    str(FIXTURE / "ConditionalFieldWritesNon01.class")])
        if jadx.returncode:
            raise RuntimeError(f"jadx failed:\n{jadx.stdout}{jadx.stderr}")
        jadx_source = jadx_dir / "sources/defpackage/ConditionalFieldWrites.java"
        shutil.copy2(jadx_source, EVIDENCE / "jadx-patched.java")
        compiled = run(["javac", "--release", "8", "-Xlint:-options", str(jadx_source)])
        (EVIDENCE / "jadx-patched-javac.log").write_text(compiled.stdout + compiled.stderr)
        if compiled.returncode == 0:
            raise AssertionError("JADX patched source unexpectedly compiled")

        print("original:   " + original_output, end="")
        print("patched:    " + patched_output, end="")
        print("Jarde outputs match original JVM outputs; JADX patched javac fails as expected")


if __name__ == "__main__":
    main()
