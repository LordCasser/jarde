#!/usr/bin/env python3
"""Rebuild, patch, and compare the Java/JADX/Jarde boolean-array fixtures.

All generated files go below the caller-provided --out directory.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path


HERE = Path(__file__).resolve().parent
FIXTURES = HERE / "fixtures"


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run(command: list[str], log_path: Path, cwd: Path | None = None) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(command, cwd=cwd, text=True, capture_output=True, check=False)
    log_path.parent.mkdir(parents=True, exist_ok=True)
    log_path.write_text(
        json.dumps(
            {
                "command": command,
                "cwd": str(cwd or Path.cwd()),
                "exit_status": result.returncode,
                "stdout": result.stdout,
                "stderr": result.stderr,
            },
            indent=2,
        )
        + "\n",
        encoding="utf-8",
    )
    return result


def require_success(result: subprocess.CompletedProcess[str], stage: str) -> None:
    if result.returncode != 0:
        raise SystemExit(f"{stage} failed with status {result.returncode}; see its JSON log")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", required=True, type=Path)
    parser.add_argument("--jarde-cli", required=True)
    parser.add_argument("--jadx", default="jadx")
    args = parser.parse_args()
    out = args.out.resolve()
    out.mkdir(parents=True, exist_ok=True)
    original = out / "classes" / "original"
    patched = out / "classes" / "patched"
    runners = out / "classes" / "runners"
    for directory in (original, patched, runners):
        directory.mkdir(parents=True, exist_ok=True)

    versions = {}
    for name, command in (
        ("java", ["java", "-version"]),
        ("javac", ["javac", "-version"]),
        ("jadx", [args.jadx, "--version"]),
        ("jarde_cli", [args.jarde_cli, "--version"]),
    ):
        version_result = run(command, out / "logs" / f"version-{name}.json")
        versions[name] = {
            "command": command,
            "exit_status": version_result.returncode,
            "stdout": version_result.stdout.strip(),
            "stderr": version_result.stderr.strip(),
            "executable": shutil.which(command[0]),
        }

    # Compile the two ordinary Java 8 int[] sources to isolated original class files.
    compile_original = run(
        ["javac", "--release", "8", "-g:none", "-d", str(original),
         str(FIXTURES / "RawBool.java"), str(FIXTURES / "Order.java")],
        out / "logs" / "javac-original.json",
    )
    require_success(compile_original, "javac original fixtures")

    patches = {}
    for name in ("RawBool", "Order"):
        source_class = original / f"{name}.class"
        destination = patched / f"{name}.class"
        manifest = out / "patches" / f"{name}.json"
        result = run(
            [sys.executable, str(HERE / "patch_classfiles.py"), name,
             "--source", str(source_class), "--destination", str(destination),
             "--manifest", str(manifest)],
            out / "logs" / f"patch-{name}.json",
        )
        require_success(result, f"patch {name}")
        patches[name] = json.loads(manifest.read_text(encoding="utf-8"))

    javap_logs = {}
    for name in ("RawBool", "Order"):
        for variant, class_root in (("original", original), ("patched", patched)):
            javap_result = run(
                ["javap", "-classpath", str(class_root), "-c", "-s", name],
                out / "logs" / f"javap-{variant}-{name}.json",
            )
            require_success(javap_result, f"javap {variant} {name}")
            javap_logs[f"{variant}-{name}"] = str(out / "logs" / f"javap-{variant}-{name}.json")

    # Compile reflection-only runners against the patched descriptors.
    for runner in ("RawBoolRunner", "OrderRunner"):
        source_class = "RawBool" if runner == "RawBoolRunner" else "Order"
        compile_runner = run(
            ["javac", "--release", "8", "-g:none", "-cp", str(patched),
             "-d", str(runners), str(FIXTURES / f"{runner}.java")],
            out / "logs" / f"javac-{runner}.json",
        )
        require_success(compile_runner, f"javac {runner}")
        runtime = run(
            ["java", "-Xverify:all", "-cp", os.pathsep.join((str(patched), str(runners)),), runner],
            out / "logs" / f"jvm-{source_class}.json",
        )
        require_success(runtime, f"java -Xverify:all {source_class}")

    stages = {}
    for name in ("RawBool", "Order"):
        jadx_dir = out / "jadx" / name
        jadx_result = run(
            [args.jadx, "-d", str(jadx_dir), str(patched / f"{name}.class")],
            out / "logs" / f"jadx-{name}.json",
        )
        jadx_source = jadx_dir / "sources" / "defpackage" / f"{name}.java"
        if not jadx_source.exists():
            raise SystemExit(f"JADX produced no expected source: {jadx_source}")
        jadx_compile_dir = jadx_dir / "javac-classes"
        jadx_compile_dir.mkdir(parents=True, exist_ok=True)
        jadx_compile = run(
            ["javac", "--release", "8", "-g:none", "-d", str(jadx_compile_dir), str(jadx_source)],
            out / "logs" / f"javac-jadx-{name}.json",
        )
        jarde_result = run(
            [args.jarde_cli, "class-source", "--input", str(patched / f"{name}.class"),
             "--class", name, "--policy", "single-class", "--evidence", "all"],
            out / "logs" / f"jarde-{name}.json",
        )
        stages[name] = {
            "jadx": {
                "exit_status": jadx_result.returncode,
                "source": str(jadx_source),
                "source_sha256": sha256(jadx_source),
                "javac_exit_status": jadx_compile.returncode,
                "javac_log": str(out / "logs" / f"javac-jadx-{name}.json"),
            },
            "jarde": {
                "exit_status": jarde_result.returncode,
                "log": str(out / "logs" / f"jarde-{name}.json"),
                "stdout_sha256": hashlib.sha256(jarde_result.stdout.encode()).hexdigest(),
                "stderr_sha256": hashlib.sha256(jarde_result.stderr.encode()).hexdigest(),
                "bytecode_markers": jarde_result.stdout.count("@bytecode"),
            },
        }

    summary = {
        "java_classfile_target": 52,
        "versions": versions,
        "cli_sha256": sha256(Path(args.jarde_cli).resolve()),
        "fixtures": patches,
        "javap_logs": javap_logs,
        "stages": stages,
        "jvm_logs": {
            "RawBool": str(out / "logs" / "jvm-RawBool.json"),
            "Order": str(out / "logs" / "jvm-Order.json"),
        },
        "all_outputs_are_under_caller_supplied_out": True,
    }
    (out / "summary.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
