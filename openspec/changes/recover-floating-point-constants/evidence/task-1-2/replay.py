#!/usr/bin/env python3
"""Replay frozen task 1.2 constant-pool and Code attribute experiments."""

from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import subprocess
import tempfile
from difflib import unified_diff
from pathlib import Path


ROOT = Path(__file__).resolve().parents[5]
CHANGE = ROOT / "openspec/changes/recover-floating-point-constants"
SOURCE_EVIDENCE = ROOT / "openspec/evidence/java-syntax-2026-09-22/floating-constants"
FIXTURE = ROOT / "tests/fixtures/p3-floating-constants/v8/FloatingConstants.class"
HELPER_SOURCES = ("FloatingSupport.java", "FloatingRunner.java")
BASE_SHA256 = "7d557979522ebda315a93715c37ba50b480f6780eb8c4be86ae6a14e540b9652"
DEFAULT_OUT = CHANGE / "evidence/task-1-2"
HISTORICAL_JARDE = {
    "frozen-original": SOURCE_EVIDENCE / "fixture/jarde.java.txt",
    "positive-quiet": SOURCE_EVIDENCE / "nan-payloads/positive-quiet/jarde.java.txt",
    "negative-quiet": SOURCE_EVIDENCE / "nan-payloads/negative-quiet/jarde.java.txt",
    "positive-signaling": SOURCE_EVIDENCE / "nan-payloads/positive-signaling/jarde.java.txt",
    "canonical-nan-fneg-dneg": SOURCE_EVIDENCE / "nan-negation/jarde.java.txt",
}


def run(args: list[str], *, cwd: Path | None = None, timeout: int = 60):
    return subprocess.run(args, cwd=cwd, capture_output=True, text=True, timeout=timeout)


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def code_attribute(code: bytes, max_stack: int) -> bytes:
    # Code attribute: 2 max_stack, 2 max_locals, 4 code_length, code, 2/2/2 empty tables.
    return (len(code) + 12).to_bytes(4, "big") + max_stack.to_bytes(2, "big") + b"\0\0" + len(code).to_bytes(4, "big") + code + b"\0\0\0\0"


def replace_code(data: bytes, before: bytes, after: bytes, max_stack: int, new_max_stack: int | None = None) -> bytes:
    old = code_attribute(before, max_stack)
    new = code_attribute(after, max_stack if new_max_stack is None else new_max_stack)
    if data.count(old) != 1:
        raise AssertionError(f"expected exactly one Code attribute for {before.hex()}, found {data.count(old)}")
    return data.replace(old, new)


def save_run(path: Path, args: list[str], cwd: Path | None = None) -> int:
    result = run(args, cwd=cwd)
    path.write_text(result.stdout + result.stderr)
    return result.returncode


def source_with_helpers(source: str, directory: Path, stem: str, evidence: Path) -> tuple[int, str]:
    package_line = next((line for line in source.splitlines() if line.startswith("package ")), "")
    pkg = package_line[len("package "):].strip().removesuffix(";") if package_line else ""
    helper_dir = directory / f"{stem}-helpers"
    helper_dir.mkdir()
    for name in HELPER_SOURCES:
        body = (SOURCE_EVIDENCE / name).read_text()
        (helper_dir / name).write_text((package_line + "\n" if package_line else "") + body)
    source_path = helper_dir / "FloatingConstants.java"
    source_path.write_text(source)
    classes = directory / f"{stem}-classes"
    classes.mkdir()
    log = evidence / f"{stem}-javac.log"
    result = run(["javac", "--release", "8", "-d", str(classes), str(source_path), *(str(helper_dir / n) for n in HELPER_SOURCES)])
    log.write_text(result.stdout + result.stderr)
    if result.returncode:
        return result.returncode, f"compile exit={result.returncode}; see {log.name}"
    runner = (pkg + "." if pkg else "") + "FloatingRunner"
    execution = run(["java", "-Xverify:all", "-cp", str(classes), runner], cwd=classes)
    (evidence / f"{stem}-execution.txt").write_text(execution.stdout + execution.stderr)
    (evidence / f"{stem}-execution-status.txt").write_text(f"exit={execution.returncode}\n")
    return execution.returncode, f"compile exit=0; execution exit={execution.returncode}"


def make_variants(base: bytes) -> list[tuple[str, bytes, str]]:
    variants: list[tuple[str, bytes, str]] = []
    for name, fbits, dbits in (
        ("positive-quiet", 0x7FC12345, 0x7FF8123456789ABC),
        ("negative-quiet", 0xFFC12345, 0xFFF8123456789ABC),
        ("positive-signaling", 0x7F812345, 0x7FF0123456789ABC),
        ("negative-signaling", 0xFF812345, 0xFFF0123456789ABC),
    ):
        float_entry = bytes.fromhex("04 7fc00000")
        double_entry = bytes.fromhex("06 7ff8000000000000")
        if base.count(float_entry) != 1 or base.count(double_entry) != 1:
            raise AssertionError("frozen class does not contain one canonical Float and Double CP entry")
        patched = base.replace(float_entry, b"\x04" + fbits.to_bytes(4, "big"))
        patched = patched.replace(double_entry, b"\x06" + dbits.to_bytes(8, "big"))
        variants.append((name, patched, f"pool Float=0x{fbits:08x}, Double=0x{dbits:016x}"))

    fneg = replace_code(base, bytes.fromhex("120fae"), bytes.fromhex("120f76ae"), 1)
    fneg = replace_code(fneg, bytes.fromhex("140022af"), bytes.fromhex("14002277af"), 2)
    variants.append(("canonical-nan-fneg-dneg", fneg, "exact Code patch inserts fneg/dneg after canonical NaN ldc/ldc2_w"))

    zero_div = replace_code(base, bytes.fromhex("120fae"), bytes.fromhex("0b0b6eae"), 1, 2)
    zero_div = replace_code(zero_div, bytes.fromhex("140022af"), bytes.fromhex("0e0e6faf"), 2, 4)
    variants.append(("runtime-zero-div-zero", zero_div, "exact Code patch replaces canonical NaN ldc with fconst_0/fdiv and dconst_0/ddiv"))
    return variants


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cli", type=Path, default=ROOT / "target/debug/jarde-cli")
    parser.add_argument("--out", type=Path, default=DEFAULT_OUT)
    args = parser.parse_args()
    out = args.out.resolve()
    cli_path = args.cli.resolve()
    out.mkdir(parents=True, exist_ok=True)
    versions = []
    for command in (["java", "-version"], ["javac", "-version"], ["javap", "-version"], ["jadx", "--version"]):
        result = run(list(command))
        versions.append(f"$ {' '.join(command)}\nexit={result.returncode}\n{result.stdout}{result.stderr}")
    (out / "tool-versions.txt").write_text("\n".join(versions))
    base = FIXTURE.read_bytes()
    if digest(base) != BASE_SHA256:
        raise SystemExit(f"frozen input hash mismatch: {digest(base)}")
    (out / "frozen-input.sha256").write_text(f"{BASE_SHA256}  FloatingConstants.class\n")
    source_dir = out / "frozen-java-input"
    source_dir.mkdir(exist_ok=True)
    for name in ("FloatingConstants.java", *HELPER_SOURCES):
        shutil.copy2(SOURCE_EVIDENCE / name, source_dir / name)
    (out / "patch-method.md").write_text(
        """# Frozen patch method\n\n"
        "Input is the permanent Java 8 class at `tests/fixtures/p3-floating-constants/v8/FloatingConstants.class`; "
        "its SHA-256 is checked before any experiment. `replay.py` also recompiles the checked-in source-only "
        "Java inputs and requires byte-for-byte equality with that class. Pool variants replace only the exact "
        "Float/Double entries including their tags. Operation variants replace exactly one complete Code "
        "attribute per method, with asserted original bytes and updated `max_stack` where needed. All generated "
        "classes live in a temporary directory and are removed when replay exits.\n\n"
        "This is fixture/方案 evidence. It does not assert that current Jarde recovery restores these values.\n"""
    )
    baseline_sources = [SOURCE_EVIDENCE / "FloatingConstants.java", *(SOURCE_EVIDENCE / n for n in HELPER_SOURCES)]
    with tempfile.TemporaryDirectory(prefix="jarde-task-1-2-") as tmp:
        work = Path(tmp)
        compiled = work / "compiled"
        compiled.mkdir()
        compile_original = run(["javac", "--release", "8", "-g:none", "-d", str(compiled), *(str(p) for p in baseline_sources)])
        (out / "source-input-javac.log").write_text(compile_original.stdout + compile_original.stderr)
        if compile_original.returncode:
            raise SystemExit(f"source-only frozen fixture failed to compile: {compile_original.returncode}")
        compiled_base = (compiled / "FloatingConstants.class").read_bytes()
        if compiled_base != base:
            raise SystemExit("source-only inputs no longer compile to the frozen permanent class")

        variants = [("frozen-original", base, "unmodified frozen class"), *make_variants(base)]
        summary = []
        for name, class_bytes, patch in variants:
            dest = out / name
            dest.mkdir(exist_ok=True)
            class_path = work / name / "FloatingConstants.class"
            class_path.parent.mkdir()
            class_path.write_bytes(class_bytes)
            for helper in HELPER_SOURCES:
                shutil.copy2(compiled / helper.replace(".java", ".class"), class_path.parent / helper.replace(".java", ".class"))
            (dest / "class-sha256.txt").write_text(f"{digest(class_bytes)}  FloatingConstants.class ({len(class_bytes)} bytes)\n")
            (dest / "patch.txt").write_text(patch + "\n")
            original_status = save_run(dest / "original.txt", ["java", "-Xverify:all", "-cp", str(class_path.parent), "FloatingRunner"], class_path.parent)
            (dest / "original-status.txt").write_text(f"exit={original_status}\n")
            save_run(dest / "javap.txt", ["javap", "-p", "-c", "-v", str(class_path)])

            jadx_dir = work / f"{name}-jadx"
            jadx_status = save_run(dest / "jadx.log", ["jadx", "--no-res", "-d", str(jadx_dir), str(class_path)])
            jadx_sources = list(jadx_dir.rglob("FloatingConstants.java")) if jadx_dir.exists() else []
            jadx_compile = "not run: JADX failed or emitted no FloatingConstants.java"
            if jadx_status == 0 and len(jadx_sources) == 1:
                jadx_source = jadx_sources[0].read_text()
                (dest / "jadx.java.txt").write_text(jadx_source)
                code, jadx_compile = source_with_helpers(jadx_source, work, f"{name}-jadx", dest)
                (dest / "jadx-compile-execution-status.txt").write_text(jadx_compile + "\n")
                if code == 0:
                    (dest / "jadx-execution.txt").write_text((dest / f"{name}-jadx-execution.txt").read_text())
            else:
                (dest / "jadx-compile-execution-status.txt").write_text(jadx_compile + f"; jadx exit={jadx_status}; sources={len(jadx_sources)}\n")
            if (dest / "jadx-execution.txt").is_file():
                difference = list(unified_diff(
                    (dest / "original.txt").read_text().splitlines(),
                    (dest / "jadx-execution.txt").read_text().splitlines(),
                    fromfile="original",
                    tofile="jadx",
                    lineterm="",
                ))
                (dest / "original-jadx-differences.txt").write_text("\n".join(difference) + "\n" if difference else "No output differences.\n")
            else:
                (dest / "original-jadx-differences.txt").write_text("JADX source did not compile and execute; see jadx status and javac log.\n")

            historical_status = None
            if cli_path.is_file():
                cli = run([str(cli_path), "class-source", "--input", str(class_path), "--class", "FloatingConstants", "--policy", "single-class", "--release", "8", "--format", "text", "--evidence", "all"])
                (dest / "jarde.java.txt").write_text(cli.stdout)
                (dest / "jarde-report.txt").write_text(f"exit={cli.returncode}\n" + cli.stderr)
                jarde_compile = source_with_helpers(cli.stdout, work, f"{name}-jarde", dest) if cli.returncode == 0 else (cli.returncode, f"not run: Jarde CLI exit={cli.returncode}")
                jarde_exit = cli.returncode
            else:
                (dest / "jarde.java.txt").unlink(missing_ok=True)
                (dest / "jarde-report.txt").write_text(f"unavailable: expected executable {cli_path}; Cargo was intentionally not run\n")
                jarde_compile = (127, f"not run: missing executable {cli_path}; Cargo was intentionally not run")
                jarde_exit = 127
                historical = HISTORICAL_JARDE.get(name)
                if historical is not None:
                    historical_source = historical.read_text()
                    shutil.copy2(historical, dest / "jarde-historical.java.txt")
                    historical_result = source_with_helpers(historical_source, work, f"{name}-historical-jarde", dest)
                    historical_status = historical_result[1]
                    (dest / "jarde-historical-compile-execution-status.txt").write_text(historical_result[1] + "\n")
                    if historical_result[0] == 0:
                        (dest / "jarde-historical-execution.txt").write_text((dest / f"{name}-historical-jarde-execution.txt").read_text())
                    (dest / "jarde-historical-provenance.txt").write_text(
                        "Copied from openspec/evidence/java-syntax-2026-09-22/floating-constants/ "
                        "for an earlier run of the same SHA-256 class bytes. This is historical fixture evidence, "
                        "not a fresh invocation of the currently unavailable Jarde executable.\n"
                    )
            (dest / "jarde-compile-execution-status.txt").write_text(jarde_compile[1] + "\n")
            if jarde_compile[0] == 0:
                (dest / "jarde-execution.txt").write_text((dest / f"{name}-jarde-execution.txt").read_text())

            original_lines = (dest / "original.txt").read_text().splitlines()
            summary.append({
                "variant": name,
                "patch": patch,
                "sha256": digest(class_bytes),
                "bytes": len(class_bytes),
                "original_java_xverify_exit": original_status,
                "original_output_lines": len(original_lines),
                "jadx_exit": jadx_status,
                "jadx_compile_execution": jadx_compile,
                "jarde_exit": jarde_exit,
                "jarde_compile_execution": jarde_compile[1],
                "historical_jarde_compile_execution": historical_status,
                "jadx_execution_diff_lines": sum(1 for line in (dest / "original-jadx-differences.txt").read_text().splitlines() if line.startswith(("+", "-")) and not line.startswith(("+++", "---"))),
            })
    (out / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
