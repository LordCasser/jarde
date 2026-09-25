#!/usr/bin/env python3
"""Freeze a verifier-valid $VALUES access and compare full Java 8 classes."""
from __future__ import annotations

import hashlib
import json
import os
import re
import subprocess
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
CLI = Path(os.environ.get("JARDE_CLI", "/tmp/jarde-generic-accepted-cli"))


def command(*argv: str, cwd: Path | None = None) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(argv, cwd=cwd, capture_output=True, text=True, timeout=90)
    if result.returncode:
        raise RuntimeError(f"{argv}: exit={result.returncode}\n{result.stdout}{result.stderr}")
    return result


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def replay(mode: str, root: Path) -> dict[str, str]:
    source = root / "input"
    debug = "-g" if mode == "debug" else "-g:none"
    command("javac", "--release", "8", debug, "-d", str(source),
            str(HERE / "E.java"), str(HERE / "Runner.java"))
    enum_file = source / "E.class"
    javap = command("javap", "-v", "-c", "-p", str(enum_file)).stdout
    field_index = int(re.search(r"#(\d+) = Fieldref\s+[^\n]*// E\.\$VALUES:\[LE;", javap).group(1))
    method_index = int(re.search(r"#(\d+) = Methodref\s+[^\n]*// E\.values:\(\)\[LE;", javap).group(1))
    original = enum_file.read_bytes()
    before = bytes((0xb8, method_index >> 8, method_index & 255, 0xb0))
    after = bytes((0xb2, field_index >> 8, field_index & 255, 0xb0))
    if original.count(before) != 1:
        raise RuntimeError("the raw() method pattern is absent or ambiguous")
    enum_file.write_bytes(original.replace(before, after))
    patched_javap = command("javap", "-v", "-c", "-p", str(enum_file)).stdout
    patched_javap = re.sub(r"^Classfile .*E\.class$", "Classfile E.class", patched_javap,
                           count=1, flags=re.MULTILINE)
    patched_javap = re.sub(r"^  Last modified .*; size ", "  Last modified <omitted>; size ",
                           patched_javap, count=1, flags=re.MULTILINE)
    (root / "E.javap.txt").write_text(patched_javap)
    original_output = command("java", "-Xverify:all", "-cp", str(source), "Runner").stdout
    if original_output != "first=B,raw=B\n":
        raise RuntimeError(f"unexpected original JVM result: {original_output!r}")

    jadx_root = root / "jadx"
    command("jadx", "-d", str(jadx_root), str(enum_file), str(source / "Runner.class"))
    jadx_sources = sorted((jadx_root / "sources").rglob("*.java"))
    if sorted(path.stem for path in jadx_sources) != ["E", "Runner"]:
        raise RuntimeError(f"unexpected JADX source set: {jadx_sources}")
    command("javac", "--release", "8", "-g:none", "-d", str(root / "jadx-classes"),
            *(str(path) for path in jadx_sources))
    jadx_output = command("java", "-Xverify:all", "-cp", str(root / "jadx-classes"),
                          "defpackage.Runner").stdout
    if jadx_output != "first=A,raw=A\n":
        raise RuntimeError(f"unexpected JADX JVM result: {jadx_output!r}")
    jadx_enum = next(path for path in jadx_sources if path.stem == "E")
    (root / "E.jadx.java").write_text(jadx_enum.read_text())

    if not CLI.is_file():
        raise RuntimeError(f"frozen Jarde CLI missing: {CLI}")
    jarde = command(str(CLI), "class-source", "--input", str(enum_file), "--class", "E",
                    "--policy", "single-class", "--release", "8", "--format", "json")
    jarde_report = json.loads(jarde.stdout)
    jarde_text = jarde_report["text"]
    (root / "E.jarde.java").write_text(jarde_text)
    if "return E.$VALUES;" not in jarde_text:
        raise RuntimeError("Jarde no longer preserves the raw backing-array read")
    jarde_source = root / "jarde-src" / "E.java"
    jarde_source.parent.mkdir()
    jarde_source.write_text(jarde_text)
    jarde_compile = subprocess.run(
        ("javac", "--release", "8", "-g:none", "-d", str(root / "jarde-classes"),
         str(jarde_source), str(HERE / "Runner.java")),
        capture_output=True, text=True, timeout=90,
    )
    jarde_log = (jarde_compile.stdout + jarde_compile.stderr).replace(str(jarde_source), "E.java")
    (root / "jarde-javac.log").write_text(jarde_log)
    if jarde_compile.returncode == 0:
        raise RuntimeError("the frozen Jarde enum unexpectedly compiled")
    return {
        "mode": mode,
        "input_E_sha256": sha(enum_file),
        "input_Runner_sha256": sha(source / "Runner.class"),
        "original_stdout": original_output,
        "jadx_stdout": jadx_output,
        "jarde_raw_read_preserved": "return E.$VALUES;" in jarde_text,
        "jarde_javac_exit": jarde_compile.returncode,
        "patch": f"raw() invokestatic #{method_index} -> getstatic #{field_index}, same 3-byte width",
    }


def main() -> None:
    results = []
    with tempfile.TemporaryDirectory(prefix="jarde-enum-values-", dir="/private/tmp") as temp:
        root = Path(temp)
        for mode in ("debug", "no-debug"):
            case = root / mode
            case.mkdir()
            results.append(replay(mode, case))
            # Keep only text facts, never compiler products or JADX caches in the repository.
            for name in ("E.javap.txt", "E.jadx.java", "E.jarde.java", "jarde-javac.log"):
                (HERE / f"{mode}.{name}").write_text((case / name).read_text())
    (HERE / "results.json").write_text(json.dumps(results, indent=2) + "\n")
    print(json.dumps(results, indent=2))


if __name__ == "__main__":
    main()
