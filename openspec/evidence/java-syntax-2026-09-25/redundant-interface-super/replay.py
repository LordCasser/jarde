#!/usr/bin/env python3
"""Replay the interface-super selection evidence without leaving build outputs behind."""

from __future__ import annotations

import hashlib
import os
import shutil
import subprocess
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parent
JARDE = Path(os.environ.get("JARDE_CLI", "/tmp/jarde-cli-audit-baseline"))
JADX = Path(os.environ.get("JADX", "/opt/homebrew/bin/jadx"))


def run(args: list[str], *, check: bool = True, cwd: Path | None = None) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(args, cwd=cwd, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    if check and result.returncode != 0:
        raise RuntimeError(
            f"command failed ({result.returncode}): {' '.join(args)}\n"
            f"stdout:\n{result.stdout}\nstderr:\n{result.stderr}"
        )
    return result


def write_result(
    path: Path,
    result: subprocess.CompletedProcess[str],
    temporary_root: Path | None = None,
) -> None:
    text = result.stdout + result.stderr
    if temporary_root is not None:
        text = text.replace(str(temporary_root), "<TMP>")
    # javap embeds the temporary class file's filesystem mtime, which is not class evidence.
    text = "\n".join(line for line in text.splitlines() if not line.lstrip().startswith("Last modified")) + "\n"
    path.write_text(text, encoding="utf-8")


def cp_utf8(entries: dict[int, tuple], index: int) -> str:
    entry = entries[index]
    if entry[0] != "utf8":
        raise ValueError(f"constant pool entry #{index} is not Utf8")
    return entry[1]


def replace_interface_owner(class_bytes: bytes, old_owner: str, new_owner: str, method: str) -> bytes:
    """Retarget one InterfaceMethodref owner while keeping bytecode and class flags unchanged."""
    data = bytearray(class_bytes)
    if data[:4] != b"\xca\xfe\xba\xbe":
        raise ValueError("not a class file")
    count = int.from_bytes(data[8:10], "big")
    entries: dict[int, tuple] = {}
    offset = 10
    index = 1
    while index < count:
        start = offset
        tag = data[offset]
        offset += 1
        if tag == 1:
            length = int.from_bytes(data[offset : offset + 2], "big")
            offset += 2
            value = bytes(data[offset : offset + length]).decode("utf-8")
            offset += length
            entries[index] = ("utf8", value, start)
        elif tag in (3, 4):
            offset += 4
        elif tag in (5, 6):
            offset += 8
            index += 1
        elif tag in (7, 8, 16, 19, 20):
            name_index = int.from_bytes(data[offset : offset + 2], "big")
            offset += 2
            entries[index] = ("single", tag, name_index, start)
        elif tag in (9, 10, 11, 12, 17, 18):
            left = int.from_bytes(data[offset : offset + 2], "big")
            right = int.from_bytes(data[offset + 2 : offset + 4], "big")
            offset += 4
            entries[index] = ("pair", tag, left, right, start)
        elif tag == 15:
            offset += 3
        else:
            raise ValueError(f"unknown constant pool tag {tag} at entry #{index}")
        index += 1

    class_name_by_index: dict[int, str] = {}
    class_index_by_name: dict[str, int] = {}
    for cp_index, entry in entries.items():
        if entry[0] == "single" and entry[1] == 7:
            name = cp_utf8(entries, entry[2])
            class_name_by_index[cp_index] = name
            class_index_by_name[name] = cp_index

    matches: list[tuple[int, int]] = []
    for cp_index, entry in entries.items():
        if entry[0] != "pair" or entry[1] != 11:
            continue
        owner_index, name_and_type_index = entry[2], entry[3]
        name_and_type = entries[name_and_type_index]
        member_name = cp_utf8(entries, name_and_type[2])
        if class_name_by_index.get(owner_index) == old_owner and member_name == method:
            matches.append((cp_index, entry[4]))
    if len(matches) != 1:
        raise ValueError(f"expected one {old_owner}.{method} InterfaceMethodref, got {len(matches)}")
    if new_owner not in class_index_by_name:
        raise ValueError(f"class constant for {new_owner} is absent")
    _, entry_offset = matches[0]
    data[entry_offset + 1 : entry_offset + 3] = class_index_by_name[new_owner].to_bytes(2, "big")
    return bytes(data)


def compile_java(
    javac: str,
    sources: Path | list[Path],
    output: Path,
    classpath: Path | None = None,
    debug: str = "none",
) -> None:
    output.mkdir(parents=True, exist_ok=True)
    args = [javac, "--release", "8", "-g" if debug == "all" else "-g:none", "-Xlint:-options"]
    if classpath is not None:
        args.extend(["-cp", str(classpath)])
    args.extend(["-d", str(output)])
    args.extend(str(source) for source in (sources if isinstance(sources, list) else [sources]))
    run(args)


def class_source(cli: Path, input_path: Path, class_name: str, policy: str = "plain-jar") -> tuple[str, str]:
    result = run([
        str(cli), "class-source", "--input", str(input_path), "--class", class_name,
        "--policy", policy, "--release", "8", "--format", "text",
    ])
    return result.stdout, result.stderr


def save_hashes() -> None:
    rows = []
    for path in sorted(p for p in ROOT.rglob("*") if p.is_file() and p.name != "SHA256SUMS.txt"):
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        rows.append(f"{digest}  {path.relative_to(ROOT).as_posix()}")
    (ROOT / "SHA256SUMS.txt").write_text("\n".join(rows) + "\n", encoding="utf-8")


def main() -> None:
    for tool in (JARDE, JADX):
        if not tool.is_file():
            raise SystemExit(f"required executable not found: {tool}")
    javac = shutil.which("javac")
    java = shutil.which("java")
    javap = shutil.which("javap")
    jar = shutil.which("jar")
    if not all((javac, java, javap, jar)):
        raise SystemExit("javac, java, javap and jar must be on PATH")

    with tempfile.TemporaryDirectory(prefix="jarde-interface-super-") as temporary:
        temp = Path(temporary)
        pos_source = ROOT / "InterfaceSuperProbe.java"
        pos_runner = ROOT / "InterfaceSuperRunner.java"
        neg_source = ROOT / "RedundantDirectSuperProbe.java"
        neg_runner = ROOT / "RedundantRunner.java"

        pos_none = temp / "positive-none"
        pos_debug = temp / "positive-debug"
        compile_java(javac, [pos_source, pos_runner], pos_none, debug="none")
        compile_java(javac, [pos_source, pos_runner], pos_debug, debug="all")
        original_none = run([java, "-Xverify:all", "-cp", str(pos_none), "InterfaceSuperRunner"]).stdout
        original_debug = run([java, "-Xverify:all", "-cp", str(pos_debug), "InterfaceSuperRunner"]).stdout
        if original_none != "11\n22\n33\n" or original_debug != original_none:
            raise AssertionError(f"unexpected original output: {original_none!r}, {original_debug!r}")
        (ROOT / "positive-original-run.txt").write_text(original_none, encoding="utf-8")
        write_result(
            ROOT / "positive-javap.txt",
            run([javap, "-classpath", str(pos_none), "-p", "-c", "-v", "InterfaceSuperProbe"]),
            temp,
        )

        pos_jar = temp / "positive.jar"
        run([jar, "cf", str(pos_jar), "-C", str(pos_none), "."])
        pos_debug_jar = temp / "positive-debug.jar"
        run([jar, "cf", str(pos_debug_jar), "-C", str(pos_debug), "."])
        jadx_dir = temp / "jadx-positive"
        run([str(JADX), "-d", str(jadx_dir), "--no-res", str(pos_jar)])
        jadx_file = jadx_dir / "sources" / "defpackage" / "InterfaceSuperProbe.java"
        jadx_raw = jadx_file.read_text(encoding="utf-8")
        (ROOT / "positive-jadx-raw.java").write_text(jadx_raw, encoding="utf-8")
        jadx_compile_source = temp / "InterfaceSuperProbe.java"
        jadx_compile_source.write_text("\n".join(line for line in jadx_raw.splitlines() if line != "package defpackage;") + "\n", encoding="utf-8")
        (ROOT / "positive-jadx-compile-source.java").write_text(jadx_compile_source.read_text(encoding="utf-8"), encoding="utf-8")
        jadx_classes = temp / "positive-jadx-classes"
        compile_java(javac, jadx_compile_source, jadx_classes, classpath=pos_none)
        jadx_run = run([java, "-Xverify:all", "-cp", os.pathsep.join((str(jadx_classes), str(pos_none))), "InterfaceSuperRunner"]).stdout
        (ROOT / "positive-jadx-run.txt").write_text(jadx_run, encoding="utf-8")

        jarde_text, jarde_report = class_source(JARDE, pos_jar, "InterfaceSuperProbe")
        (ROOT / "positive-jarde-InterfaceSuperProbe.java").write_text(jarde_text, encoding="utf-8")
        jarde_source = temp / "InterfaceSuperProbe.java"
        jarde_source.write_text(jarde_text, encoding="utf-8")
        jarde_classes = temp / "positive-jarde-classes"
        compile_java(javac, jarde_source, jarde_classes, classpath=pos_none)
        jarde_run = run([java, "-Xverify:all", "-cp", os.pathsep.join((str(jarde_classes), str(pos_none))), "InterfaceSuperRunner"]).stdout
        (ROOT / "positive-jarde-run.txt").write_text(jarde_run, encoding="utf-8")
        report_summary = [
            line for line in jarde_report.splitlines()
            if ".outcome.report.syntax_status =" in line
            or ".outcome.report.verification =" in line
        ]
        (ROOT / "positive-jarde-report-summary.txt").write_text(
            "\n".join(report_summary) + "\n", encoding="utf-8"
        )
        if jadx_run != "7\n7\n14\n" or jarde_run != original_none:
            raise AssertionError(f"positive mismatch: original={original_none!r}; JADX={jadx_run!r}; Jarde={jarde_run!r}")

        jarde_debug_text, _ = class_source(JARDE, pos_debug_jar, "InterfaceSuperProbe")
        (ROOT / "positive-debug-jarde-InterfaceSuperProbe.java").write_text(jarde_debug_text, encoding="utf-8")
        jarde_debug_source = temp / "InterfaceSuperProbe.java"
        jarde_debug_source.write_text(jarde_debug_text, encoding="utf-8")
        jarde_debug_classes = temp / "positive-jarde-debug-classes"
        compile_java(javac, jarde_debug_source, jarde_debug_classes, classpath=pos_debug)
        jarde_debug_run = run([
            java, "-Xverify:all", "-cp", os.pathsep.join((str(jarde_debug_classes), str(pos_debug))),
            "InterfaceSuperRunner",
        ]).stdout
        (ROOT / "positive-debug-jarde-run.txt").write_text(jarde_debug_run, encoding="utf-8")
        if jarde_debug_run != original_debug or jarde_debug_text != jarde_text:
            raise AssertionError("class-source changed on debug metadata or emitted a different result")
        if hashlib.sha256((jarde_classes / "InterfaceSuperProbe.class").read_bytes()).digest() != hashlib.sha256(
            (pos_none / "InterfaceSuperProbe.class").read_bytes()
        ).digest():
            raise AssertionError("Jarde positive class differs from the javac -g:none class")

        neg_classes = temp / "negative-compiled"
        compile_java(javac, [neg_source, neg_runner], neg_classes, debug="none")
        write_result(
            ROOT / "negative-javap-before.txt",
            run([javap, "-classpath", str(neg_classes), "-p", "-c", "-v", "RedundantDirectSuperProbe"]),
            temp,
        )
        original_neg = run([java, "-Xverify:all", "-cp", str(neg_classes), "RedundantRunner"]).stdout
        if original_neg != "4\n":
            raise AssertionError(f"legal source baseline should select Child default, got {original_neg!r}")
        (ROOT / "negative-legal-original-run.txt").write_text(original_neg, encoding="utf-8")

        patched_class = replace_interface_owner(
            (neg_classes / "RedundantDirectSuperProbe.class").read_bytes(),
            "RedundantChild", "RedundantParent", "value",
        )
        patched_dir = temp / "negative-patched"
        patched_dir.mkdir()
        (patched_dir / "RedundantDirectSuperProbe.class").write_bytes(patched_class)
        write_result(ROOT / "negative-javap-patched.txt", run([
            javap, "-classpath", os.pathsep.join((str(patched_dir), str(neg_classes))),
            "-p", "-c", "-v", "RedundantDirectSuperProbe",
        ]), temp)
        patched_run = run([java, "-Xverify:all", "-cp", os.pathsep.join((str(patched_dir), str(neg_classes))), "RedundantRunner"]).stdout
        (ROOT / "negative-original-run.txt").write_text(patched_run, encoding="utf-8")
        if patched_run != "3\n":
            raise AssertionError(f"patched JVM input should select Parent default, got {patched_run!r}")

        jarde_negative, negative_jarde_report = class_source(
            JARDE, patched_dir / "RedundantDirectSuperProbe.class", "RedundantDirectSuperProbe", "single-class"
        )
        (ROOT / "negative-jarde-RedundantDirectSuperProbe.java").write_text(jarde_negative, encoding="utf-8")
        jarde_neg_src = temp / "RedundantDirectSuperProbe.java"
        jarde_neg_src.write_text(jarde_negative, encoding="utf-8")
        compile_failure = run([
            javac, "--release", "8", "-g:none", "-Xlint:-options", "-cp", str(neg_classes),
            "-d", str(temp / "negative-jarde-classes"), str(jarde_neg_src),
        ], check=False)
        write_result(ROOT / "negative-jarde-javac.txt", compile_failure, temp)
        negative_report_summary = [
            line for line in negative_jarde_report.splitlines()
            if ".outcome.report.syntax_status =" in line
            or ".outcome.report.verification =" in line
        ]
        (ROOT / "negative-jarde-report-summary.txt").write_text(
            "\n".join(negative_report_summary) + "\n", encoding="utf-8"
        )
        if compile_failure.returncode == 0 or "RedundantParent" not in compile_failure.stderr or "冗余接口" not in compile_failure.stderr:
            raise AssertionError(f"expected javac to reject Parent.super as redundant: {compile_failure.stderr}")

        indirect_failure = run([
            javac, "--release", "8", "-g:none", "-Xlint:-options", "-d", str(temp / "negative-indirect"),
            str(ROOT / "RedundantIndirectNegative.java"),
        ], check=False)
        write_result(ROOT / "negative-indirect-javac.txt", indirect_failure)
        if indirect_failure.returncode == 0:
            raise AssertionError("javac unexpectedly accepted an indirect-interface super qualifier")

        class_hashes = []
        for label, path in (
            ("positive_original_g_none", pos_none / "InterfaceSuperProbe.class"),
            ("positive_original_g", pos_debug / "InterfaceSuperProbe.class"),
            ("positive_jadx_recompiled", jadx_classes / "InterfaceSuperProbe.class"),
            ("positive_jarde_recompiled", jarde_classes / "InterfaceSuperProbe.class"),
            ("positive_jarde_debug_recompiled", jarde_debug_classes / "InterfaceSuperProbe.class"),
            ("negative_legal_child_super", neg_classes / "RedundantDirectSuperProbe.class"),
            ("negative_patched_parent_super", patched_dir / "RedundantDirectSuperProbe.class"),
        ):
            class_hashes.append(f"{hashlib.sha256(path.read_bytes()).hexdigest()}  {label}")
        (ROOT / "classfile-sha256.txt").write_text("\n".join(class_hashes) + "\n", encoding="utf-8")

        versions = []
        for command in ([java, "-version"], [javac, "-version"], [str(JADX), "--version"], [str(JARDE), "--version"]):
            result = run(list(command), check=False)
            versions.append(result.stderr or result.stdout)
        (ROOT / "tool-versions.txt").write_text("\n".join(v.strip() for v in versions) + "\n", encoding="utf-8")
        tool_hashes = [
            f"{hashlib.sha256(JADX.read_bytes()).hexdigest()}  JADX executable",
            f"{hashlib.sha256(JARDE.read_bytes()).hexdigest()}  Jarde CLI executable",
        ]
        (ROOT / "tool-binary-sha256.txt").write_text("\n".join(tool_hashes) + "\n", encoding="utf-8")

    save_hashes()
    print(f"Replay passed; temporary class/jar outputs were removed. Evidence: {ROOT}")


if __name__ == "__main__":
    main()
