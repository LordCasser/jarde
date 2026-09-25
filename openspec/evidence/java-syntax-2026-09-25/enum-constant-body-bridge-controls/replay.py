#!/usr/bin/env python3
"""Rebuild a verifier-valid enum access-bridge counterexample and compare tools."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import re
import shutil
import struct
import subprocess
import tempfile
import zipfile


HERE = Path(__file__).resolve().parent
RESULTS = HERE / "results"


def run(*args: object) -> subprocess.CompletedProcess[str]:
    return subprocess.run([str(arg) for arg in args], text=True, capture_output=True)


def checked(*args: object) -> subprocess.CompletedProcess[str]:
    result = run(*args)
    if result.returncode:
        raise RuntimeError(f"{args[0]} exited {result.returncode}: {result.stdout}{result.stderr}")
    return result


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def u2(data: bytes | bytearray, at: int) -> int:
    return struct.unpack_from(">H", data, at)[0]


def u4(data: bytes | bytearray, at: int) -> int:
    return struct.unpack_from(">I", data, at)[0]


def skip_attributes(data: bytes | bytearray, at: int, count: int) -> int:
    for _ in range(count):
        at += 6 + u4(data, at + 2)
    return at


def bridge_code(data: bytearray) -> tuple[int, int]:
    if data[:4] != b"\xca\xfe\xba\xbe" or u2(data, 6) != 52:
        raise ValueError("expected a Java 8 class file")
    utf: dict[int, str] = {}
    at = 10
    index = 1
    count = u2(data, 8)
    while index < count:
        tag = data[at]
        at += 1
        if tag == 1:
            size = u2(data, at)
            utf[index] = bytes(data[at + 2:at + 2 + size]).decode("utf-8")
            at += 2 + size
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
            raise ValueError(f"unknown constant-pool tag {tag}")
        index += 1
    at += 6  # class access, this_class, super_class
    at += 2 + 2 * u2(data, at)  # interfaces
    fields = u2(data, at)
    at += 2
    for _ in range(fields):
        at = skip_attributes(data, at + 8, u2(data, at + 6))
    methods = u2(data, at)
    at += 2
    matching: list[tuple[int, int]] = []
    for _ in range(methods):
        flags, name, descriptor, attributes = (u2(data, at + offset) for offset in (0, 2, 4, 6))
        attr_at = at + 8
        for _ in range(attributes):
            kind, length = utf[u2(data, attr_at)], u4(data, attr_at + 2)
            info = attr_at + 6
            if (utf[name], utf[descriptor]) == (
                "<init>", "(Ljava/lang/String;ILdemo/Op$1;)V"
            ) and kind == "Code" and flags == 0x1000:
                matching.append((info + 8, u4(data, info + 4)))
            attr_at += 6 + length
        at = attr_at
    if len(matching) != 1:
        raise ValueError(f"expected one synthetic access bridge, got {matching}")
    return matching[0]


def patch_ordinal(path: Path) -> tuple[str, str]:
    data = bytearray(path.read_bytes())
    before = sha(data)
    start, length = bridge_code(data)
    if length != 7 or bytes(data[start:start + 4]) != b"\x2a\x2b\x1c\xb7" or data[start + 6] != 0xb1:
        raise ValueError(f"access bridge Code shape changed: {data[start:start + length].hex()}")
    data[start + 2] = 0x04  # BCI 2: iload_2 -> iconst_1, same width and stack type.
    path.write_bytes(data)
    return before, sha(data)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--jarde-cli", type=Path, required=True)
    args = parser.parse_args()
    cli = args.jarde_cli.resolve()
    if not cli.is_file():
        raise SystemExit(f"missing CLI: {cli}")
    jadx = shutil.which("jadx")
    if jadx is None:
        raise SystemExit("jadx is unavailable")
    RESULTS.mkdir(exist_ok=True)
    javac_version = checked("javac", "-version")
    summary: dict[str, object] = {
        "tools": {
            "javac": (javac_version.stdout + javac_version.stderr).strip(),
            "jadx": checked(jadx, "--version").stdout.strip(),
            "jarde_cli_sha256": sha(cli.read_bytes()),
        },
        "variants": {},
    }
    for debug, label in (("-g", "g"), ("-g:none", "g-none")):
        with tempfile.TemporaryDirectory(prefix=f"enum-bridge-{label}-") as tmp:
            root = Path(tmp)
            original, jadx_classes, jarde_classes = (root / name for name in ("original", "jadx-classes", "jarde-classes"))
            original.mkdir()
            jadx_classes.mkdir()
            jarde_classes.mkdir()
            checked("javac", "--release", "8", "-Xlint:-options", debug, "-d", original, HERE / "Op.java", HERE / "Probe.java")
            parent = original / "demo/Op.class"
            before, after = patch_ordinal(parent)
            original_run = checked("java", "-Xverify:all", "-cp", original, "demo.Probe").stdout
            expected_original = "ADD:1=10\nMULTIPLY:1=21\n"
            if original_run != expected_original:
                raise ValueError(f"patched original behavior changed: {original_run!r}")
            javap = checked("javap", "-v", "-c", "-p", "-classpath", original, "demo.Op").stdout
            if "2: iconst_1" not in javap:
                raise ValueError("patched BCI 2 is absent from javap")
            stable_javap = re.sub(r"Last modified .*; size (\d+) bytes", r"Last modified <normalized>; size \1 bytes", javap)
            (RESULTS / f"original-javap-{label}.txt").write_text(stable_javap.replace(str(root), "<TMP>"))
            archive = root / "subject.jar"
            with zipfile.ZipFile(archive, "w") as jar:
                for path in sorted((original / "demo").glob("Op*.class")):
                    jar.write(path, path.relative_to(original))
            jadx_dir = root / "jadx"
            checked(jadx, "-d", jadx_dir, archive)
            jadx_sources = sorted((jadx_dir / "sources").rglob("*.java"))
            if len(jadx_sources) != 1 or jadx_sources[0].name != "Op.java":
                raise ValueError(f"unexpected JADX source set: {jadx_sources}")
            jadx_source = jadx_sources[0].read_text()
            (RESULTS / f"jadx-Op-{label}.java").write_text(jadx_source)
            checked("javac", "--release", "8", "-Xlint:-options", "-d", jadx_classes, *jadx_sources, HERE / "Probe.java")
            jadx_run = checked("java", "-Xverify:all", "-cp", jadx_classes, "demo.Probe").stdout
            expected_jadx = "ADD:0=10\nMULTIPLY:1=21\n"
            if jadx_run != expected_jadx:
                raise ValueError(f"JADX behavior changed: {jadx_run!r}")
            jarde_source = root / "jarde-source/demo/Op.java"
            jarde_source.parent.mkdir(parents=True)
            jarde_result = run(cli, "class-source", "--input", archive, "--class", "demo.Op", "--output", jarde_source)
            if jarde_result.returncode:
                raise ValueError(f"Jarde class-source failed: {jarde_result.stderr[-1000:]}")
            (RESULTS / f"jarde-Op-{label}.java").write_text(jarde_source.read_text())
            compile_jarde = run("javac", "--release", "8", "-Xlint:-options", "-d", jarde_classes, jarde_source, HERE / "Probe.java")
            jarde_compile_text = (compile_jarde.stdout + compile_jarde.stderr).replace(str(root), "<TMP>")
            (RESULTS / f"jarde-javac-{label}.txt").write_text(jarde_compile_text)
            if compile_jarde.returncode == 0:
                raise ValueError("the current Jarde baseline unexpectedly compiled; re-evaluate refusal")
            summary["variants"][label] = {
                "original_class_before_sha256": before,
                "patched_class_sha256": after,
                "patch": "synthetic Op.<init>(String,int,Op$1) BCI 2 iload_2 -> iconst_1",
                "original_run": original_run,
                "jadx_recompiled_run": jadx_run,
                "jarde_javac_exit": compile_jarde.returncode,
                "jarde_source_sha256": sha(jarde_source.read_bytes()),
                "jadx_source_sha256": sha(jadx_source.encode()),
            }
    (HERE / "summary.json").write_text(json.dumps(summary, ensure_ascii=False, indent=2, sort_keys=True) + "\n")
    inputs = [HERE / name for name in ("Op.java", "Probe.java", "replay.py")]
    outputs = sorted(RESULTS.iterdir()) + [HERE / "summary.json"]
    (HERE / "manifest.sha256").write_text("".join(
        f"{sha(path.read_bytes())}  {path.relative_to(HERE)}\n" for path in inputs + outputs
    ))
    print(json.dumps(summary["variants"], ensure_ascii=False, sort_keys=True))


if __name__ == "__main__":
    main()
