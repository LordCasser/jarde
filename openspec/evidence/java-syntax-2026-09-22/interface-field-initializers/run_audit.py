#!/usr/bin/env python3
"""Replay the Java 8 interface-field initializer audit with an explicit CLI."""

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import struct
import subprocess
import tempfile


HERE = Path(__file__).resolve().parent
NAMES = ("InterfaceInitProbe", "InitEffects", "InitRunner")


def run(command, log):
    result = subprocess.run(command, capture_output=True, text=True, timeout=45)
    log.write_text(result.stdout + result.stderr)
    return result.returncode, result.stdout


def u2(data, at):
    return struct.unpack_from(">H", data, at)[0]


def u4(data, at):
    return struct.unpack_from(">I", data, at)[0]


def reorder_fields(data):
    """Swap only FIRST/SECOND field_info records; leave all Code bytes untouched."""
    assert data[:4] == b"\xca\xfe\xba\xbe"
    at = 10
    names = {}
    cp_count = u2(data, 8)
    index = 1
    while index < cp_count:
        tag = data[at]
        at += 1
        if tag == 1:
            length = u2(data, at)
            at += 2
            names[index] = data[at:at + length].decode("utf-8")
            at += length
        elif tag in (3, 4, 9, 10, 11, 12, 17, 18):
            at += 4
        elif tag in (5, 6):
            at += 8
            index += 1
        elif tag in (7, 8, 16, 19, 20):
            at += 2
        elif tag == 15:
            at += 3
        else:
            raise ValueError(f"unexpected constant-pool tag {tag}")
        index += 1
    at += 6  # access_flags, this_class, super_class
    at += 2 + 2 * u2(data, at)  # interfaces
    count = u2(data, at)
    at += 2
    fields = []
    for _ in range(count):
        start = at
        name = names[u2(data, at + 2)]
        attributes = u2(data, at + 6)
        at += 8
        for _ in range(attributes):
            at += 6 + u4(data, at + 2)
        fields.append((name, start, at))
    indexes = {name: i for i, (name, _, _) in enumerate(fields)}
    assert set(("FIRST", "SECOND")) <= indexes.keys()
    first, second = indexes["FIRST"], indexes["SECOND"]
    ordered = [data[start:end] for _, start, end in fields]
    ordered[first], ordered[second] = ordered[second], ordered[first]
    changed = data[:fields[0][1]] + b"".join(ordered) + data[fields[-1][2]:]
    assert len(changed) == len(data)
    return changed


def decompile_jadx(binary, base, out):
    jadx_dir = base / "jadx"
    status, _ = run(["jadx", "--no-res", "-d", str(jadx_dir), str(binary)],
                    out / "jadx.log")
    assert status == 0
    candidates = list(jadx_dir.rglob("InterfaceInitProbe.java"))
    assert len(candidates) == 1
    generated = candidates[0]
    shutil.copy2(generated, out / "jadx.java.txt")
    package = next((line for line in generated.read_text().splitlines()
                    if line.startswith("package ")), "")
    package_name = package.removeprefix("package ").rstrip(";")
    support = base / "jadx-support"
    support.mkdir()
    for name in ("InitEffects", "InitRunner"):
        (support / f"{name}.java").write_text(
            package + "\n" + (HERE / f"{name}.java").read_text()
        )
    classes = base / "jadx-classes"
    status, _ = run(["javac", "--release", "8", "-g:none", "-d", str(classes),
                     str(generated), *(str(support / f"{name}.java")
                                      for name in ("InitEffects", "InitRunner"))],
                    out / "jadx-javac.log")
    runtime = None
    if status == 0:
        runner = f"{package_name}.InitRunner" if package_name else "InitRunner"
        runtime_status, runtime = run(["java", "-Xverify:all", "-cp", str(classes),
                                       runner], out / "jadx-runtime.txt")
        assert runtime_status == 0
    return status, runtime


def decompile_jarde(binary, cli, base, out):
    result = subprocess.run(
        [str(cli), "class-source", "--input", str(binary), "--class",
         "InterfaceInitProbe", "--policy", "single-class", "--release", "8",
         "--format", "text"],
        capture_output=True, text=True, timeout=45,
    )
    (out / "jarde.java.txt").write_text(result.stdout)
    (out / "jarde-report.txt").write_text(result.stderr)
    assert result.returncode == 0
    source = base / "jarde" / "InterfaceInitProbe.java"
    source.parent.mkdir()
    source.write_text(result.stdout)
    classes = base / "jarde-classes"
    status, _ = run(["javac", "--release", "8", "-g:none", "-d", str(classes),
                     str(source), *(str(HERE / f"{name}.java")
                                    for name in ("InitEffects", "InitRunner"))],
                    out / "jarde-javac.log")
    runtime = None
    if status == 0:
        runtime_status, runtime = run(["java", "-Xverify:all", "-cp", str(classes),
                                       "InitRunner"], out / "jarde-runtime.txt")
        assert runtime_status == 0
    return status, runtime


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--cli", type=Path, required=True)
    parser.add_argument("--out", type=Path, default=HERE)
    args = parser.parse_args()
    args.out.mkdir(parents=True, exist_ok=True)
    summary = {}
    with tempfile.TemporaryDirectory(prefix="jarde-interface-init-") as temp:
        work = Path(temp)
        original = work / "compiled"
        original.mkdir()
        status, _ = run(["javac", "--release", "8", "-g:none", "-d", str(original),
                         *(str(HERE / f"{name}.java") for name in NAMES)],
                        args.out / "original-javac.log")
        assert status == 0
        probe = original / "InterfaceInitProbe.class"
        pristine = probe.read_bytes()
        reordered = reorder_fields(pristine)
        for label, data in (("original", pristine), ("field-table-reordered", reordered)):
            case = work / label
            case.mkdir()
            out = args.out / label
            out.mkdir()
            for name in NAMES:
                (case / f"{name}.class").write_bytes(
                    data if name == "InterfaceInitProbe"
                    else (original / f"{name}.class").read_bytes()
                )
            binary = case / "InterfaceInitProbe.class"
            shutil.copy2(binary, out / "InterfaceInitProbe.class")
            code, runtime = run(["java", "-Xverify:all", "-cp", str(case),
                                 "InitRunner"], out / "original-runtime.txt")
            assert code == 0
            run(["javap", "-v", "-p", "-c", str(binary)], out / "javap.txt")
            jarde_status, jarde_runtime = decompile_jarde(binary, args.cli, case, out)
            jadx_status, jadx_runtime = decompile_jadx(binary, case, out)
            summary[label] = {
                "class_sha256": hashlib.sha256(data).hexdigest(),
                "original_runtime": runtime.strip(),
                "jadx_javac": jadx_status,
                "jadx_runtime": None if jadx_runtime is None else jadx_runtime.strip(),
                "jarde_javac": jarde_status,
                "jarde_runtime": None if jarde_runtime is None else jarde_runtime.strip(),
            }
        assert summary["original"]["original_runtime"] == summary[
            "field-table-reordered"]["original_runtime"]
    (args.out / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
