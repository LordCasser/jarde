#!/usr/bin/env python3
"""Replay the bounded DT-12 enum-body audit against Java 8 source, JADX, and Jarde."""
from __future__ import annotations

import hashlib
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile


ROOT = Path(__file__).resolve().parents[4]
EVIDENCE = Path(__file__).resolve().parent
JADX_REPO = Path("/Users/lordcasser/workspace/testzone/jadx")
JADX = JADX_REPO / "jadx-cli/build/install/jadx/bin/jadx"
EXPECTED_JADX_HEAD = "2fb1b16386941660fda07e9017285aec40fcb37f"
CASES = {
    "TestEnums2a": ("DoubleOperations", "IOps", "Runner"),
    "TestEnums6": ("Numbers", "Runner"),
}
EXPECTED = {
    "TestEnums2a": "TIMES=*:6:demo.DoubleOperations$1\nDIVIDE=/:2:demo.DoubleOperations$2\n",
    "TestEnums6": "values=ZERO:0,ONE:1\ndeclared-constructors=[2, 3]\n",
}


def run(args, *, cwd=None, env=None):
    return subprocess.run(args, cwd=cwd, env=env, text=True, capture_output=True)


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def save(path, result, temp):
    if isinstance(result, subprocess.CompletedProcess):
        value = result.stdout + result.stderr + f"exit={result.returncode}\n"
    else:
        value = str(result)
    value = value.replace(str(temp), "<TEMP>").replace(str(ROOT), "<REPO>")
    value = "\n".join(line.rstrip() for line in value.splitlines()) + "\n"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(value)


def java_sources(directory):
    return sorted(directory.rglob("*.java"))


if len(sys.argv) != 3 or sys.argv[1] not in ("baseline", "fixed"):
    raise SystemExit("usage: replay.py baseline|fixed /absolute/path/to/jarde-cli")
JARDE_EXPECTATION, JARDE = sys.argv[1], sys.argv[2]
OUTPUT = EVIDENCE if JARDE_EXPECTATION == "baseline" else EVIDENCE / "fixed"
if not Path(JARDE).is_file() or not JADX.is_file():
    raise SystemExit("Jarde CLI and JADX executable must be present")

jadx_head = run(["git", "-C", str(JADX_REPO), "rev-parse", "HEAD"])
jadx_status = run(["git", "-C", str(JADX_REPO), "status", "--porcelain"])
if jadx_head.returncode or jadx_head.stdout.strip() != EXPECTED_JADX_HEAD or jadx_status.stdout.strip():
    raise SystemExit("JADX checkout must be clean at the frozen 1.5.6 commit")
with tempfile.TemporaryDirectory(prefix="jarde-dt12-audit-") as temp_name:
    temp = Path(temp_name)
    env = os.environ.copy()
    env["LC_ALL"] = "C"
    versions = []
    for command in (("java", "-version"), ("javac", "-version"), ("javap", "-version")):
        proc = run(list(command))
        versions.append(f"$ {' '.join(command)}\n{proc.stdout}{proc.stderr}exit={proc.returncode}\n")
    jadx_version = run([str(JADX), "--version"])
    versions.append(f"$ {JADX} --version\n{jadx_version.stdout}{jadx_version.stderr}exit={jadx_version.returncode}\n")
    versions.append(
        f"JADX checkout HEAD: {jadx_head.stdout.strip()}\n"
        f"JADX checkout status: clean\n"
        f"Jarde invocation: python3 replay.py {JARDE_EXPECTATION} <JARDE_CLI>\n"
    )
    save(OUTPUT / "toolchain.txt", "".join(versions), temp)

    for case, source_names in CASES.items():
        sources = EVIDENCE / case
        variants = OUTPUT / "outputs" / case
        for debug in ("g", "g-none"):
            original_classes = temp / case / debug / "original-classes"
            original_classes.mkdir(parents=True)
            debug_args = ["-g"] if debug == "g" else ["-g:none"]
            compile_args = ["javac", "-J-Duser.language=en", "-J-Duser.country=US", "--release", "8", *debug_args, "-d", str(original_classes), *map(str, java_sources(sources))]
            original_compile = run(compile_args, env=env)
            save(variants / debug / "original-javac.log", original_compile, temp)
            if original_compile.returncode:
                raise SystemExit(f"{case}/{debug}: original Java 8 source did not compile")
            original_run = run(["java", "-Xverify:all", "-cp", str(original_classes), "demo.Runner"], env=env)
            save(variants / debug / "original-run.log", original_run, temp)
            if original_run.returncode or original_run.stdout != EXPECTED[case]:
                raise SystemExit(f"{case}/{debug}: original runtime differs")
            original_javap = run(["javap", "-v", "-c", "-p", "-classpath", str(original_classes), *[f"demo.{name}" for name in source_names if name != "Runner"]], env=env)
            save(variants / debug / "original-javap.txt", original_javap, temp)
            if original_javap.returncode:
                raise SystemExit(f"{case}/{debug}: javap failed")

            jar = temp / case / debug / "original.jar"
            packed = run(["jar", "--create", "--date=2000-01-01T00:00:00Z", "--file", str(jar), "-C", str(original_classes), "."], env=env)
            save(variants / debug / "jar.log", packed, temp)
            if packed.returncode:
                raise SystemExit(f"{case}/{debug}: jar failed")

            jadx_source = temp / case / debug / "jadx-source"
            decompiled = run([JADX, "-d", str(jadx_source), str(jar)], env=env)
            save(variants / debug / "jadx.log", decompiled, temp)
            if decompiled.returncode:
                raise SystemExit(f"{case}/{debug}: JADX failed")
            jadx_files = java_sources(jadx_source)
            stable_jadx = variants / debug / "jadx-source"
            if stable_jadx.exists():
                shutil.rmtree(stable_jadx)
            for source in jadx_files:
                target = stable_jadx / source.relative_to(jadx_source)
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_text(source.read_text().rstrip() + "\n")
            jadx_classes = temp / case / debug / "jadx-classes"
            jadx_classes.mkdir()
            jadx_compile = run(["javac", "-J-Duser.language=en", "-J-Duser.country=US", "--release", "8", *debug_args, "-d", str(jadx_classes), *map(str, jadx_files)], env=env)
            save(variants / debug / "jadx-javac.log", jadx_compile, temp)
            if jadx_compile.returncode:
                raise SystemExit(f"{case}/{debug}: complete JADX source did not compile")
            jadx_run = run(["java", "-Xverify:all", "-cp", str(jadx_classes), "demo.Runner"], env=env)
            save(variants / debug / "jadx-run.log", jadx_run, temp)
            if jadx_run.returncode or jadx_run.stdout != EXPECTED[case]:
                raise SystemExit(f"{case}/{debug}: JADX runtime differs")

            jarde_source = variants / debug / "jarde-source"
            if jarde_source.exists():
                shutil.rmtree(jarde_source)
            # Anonymous enum children should be represented inside the enum constant bodies when
            # proved. Standalone helper and runner roots remain ordinary source files.
            for name in source_names:
                output = run([JARDE, "class-source", "--input", str(jar), "--class", f"demo/{name}", "--policy", "plain-jar", "--release", "8", "--format", "text"], env=env)
                save(variants / debug / f"jarde-{name}-status.log", f"exit={output.returncode}\n", temp)
                if output.returncode:
                    raise SystemExit(f"{case}/{debug}: Jarde did not present {name}")
                target = jarde_source / "demo" / f"{name}.java"
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_text(output.stdout)
            # JSON reports contain per-run timing counters; the frozen evidence keeps stable
            # class-source text and javac diagnostics instead of volatile reports.
            jarde_files = java_sources(jarde_source)
            jarde_classes = temp / case / debug / "jarde-classes"
            jarde_classes.mkdir()
            jarde_compile = run(["javac", "-J-Duser.language=en", "-J-Duser.country=US", "--release", "8", *debug_args, "-d", str(jarde_classes), *map(str, jarde_files)], env=env)
            save(variants / debug / "jarde-javac.log", jarde_compile, temp)
            if case == "TestEnums2a":
                if JARDE_EXPECTATION == "baseline" and jarde_compile.returncode == 0:
                    raise SystemExit("baseline expectation failed: TestEnums2a unexpectedly recompiles")
                if JARDE_EXPECTATION == "baseline" and "enum constant expected" not in jarde_compile.stderr:
                    raise SystemExit("baseline expectation failed: TestEnums2a lacks the frozen enum syntax diagnostic")
                if JARDE_EXPECTATION == "fixed" and jarde_compile.returncode != 0:
                    raise SystemExit("fixed expectation failed: TestEnums2a did not recompile")
            if jarde_compile.returncode == 0:
                jarde_run = run(["java", "-Xverify:all", "-cp", str(jarde_classes), "demo.Runner"], env=env)
                save(variants / debug / "jarde-run.log", jarde_run, temp)
                if jarde_run.returncode or jarde_run.stdout != EXPECTED[case]:
                    raise SystemExit(f"{case}/{debug}: Jarde output compiled but failed verification/runtime")
            else:
                save(variants / debug / "jarde-run.log", "not run: complete Jarde source set failed javac --release 8\n", temp)

            hashes = []
            for path in sorted(original_classes.rglob("*.class")):
                hashes.append(f"{sha(path)}  {path.relative_to(original_classes)}\n")
            save(variants / debug / "original-class-sha256.txt", "".join(hashes), temp)
            if "DoubleOperations" in source_names:
                enum = (jarde_source / "demo/DoubleOperations.java").read_text()
                if "enum DoubleOperations" not in enum or "TIMES" not in enum:
                    raise SystemExit("Jarde output lost the physical enum declaration")

print("DT-12 replay complete")
