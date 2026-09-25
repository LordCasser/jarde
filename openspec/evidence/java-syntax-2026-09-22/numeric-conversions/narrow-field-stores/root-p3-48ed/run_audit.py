#!/usr/bin/env python3
"""Replay the minimized narrow-field-store audit without Cargo.

The source class is compiled afresh, then the checked-in class-file patcher changes
only field descriptors and matching Fieldref descriptors.  Every subsequent stage
uses the resulting bytes, so the JVM, jarde, and JADX observations are tied to the
same reproducible input.
"""
from __future__ import annotations

import hashlib
import json
import os
import shutil
import subprocess
import tempfile
from pathlib import Path


EVIDENCE = Path(__file__).resolve().parent
CLI = Path("/tmp/jarde-cli-narrow-field-stores-48ed")
EXPECTED_CLI_SHA = "48edb9d2e3eec451983aabb4affcbcaf6d723c8a75b0284605f729743088cc76"
EXPECTED_SOURCE_SHA = "94cbe660367772ce2bf2debc860a9c7ad4607e9b940f0176b6d8a24e8471f8cb"
EXPECTED_PATCHED_SHA = "d4797140eed55121de091518e7db4d9dcb5052d0c9a451ef6ded07e1589af8ef"
EXPECTED_SOURCE_RUNTIME_SHA = "00e009d8fda381ec5540a0fc745b8a42f04edbb7f98caa70f84d1996caeb7d6f"
EXPECTED_PATCHED_RUNTIME_SHA = "eb57ef6493d6713fbaaaf26b87e55bf5708ca8acfb671270f6cc9b6c175cc10c"

SOURCE = EVIDENCE / "NarrowFieldStores.java"
EFFECTS = EVIDENCE / "NarrowFieldStoreEffects.java"
RUNNER = EVIDENCE / "NarrowFieldStoresRunner.java"
PATCHER = EVIDENCE / "patch_field_stores.py"


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def class_major(path: Path) -> int:
    data = path.read_bytes()
    if data[:4] != b"\xca\xfe\xba\xbe":
        raise SystemExit(f"not a class file: {path}")
    return int.from_bytes(data[6:8], "big")


def record(name: str, command: list[str], stdout: str, stderr: str, status: int) -> None:
    (EVIDENCE / f"{name}.command.txt").write_text(" ".join(command) + "\n", encoding="utf-8")
    (EVIDENCE / f"{name}.stdout").write_text(stdout, encoding="utf-8")
    (EVIDENCE / f"{name}.stderr").write_text(stderr, encoding="utf-8")
    (EVIDENCE / f"{name}.status").write_text(f"{status}\n", encoding="utf-8")

    aliases = {
        "source-runtime": "source-runtime.txt",
        "patched-runtime": "patched-runtime.txt",
        "jarde-runtime": "jarde-runtime.txt",
        "jadx-runtime": "jadx-runtime.txt",
        "source-javap": "source-javap.txt",
        "patched-javap": "patched-javap.txt",
    }
    alias = aliases.get(name)
    if alias:
        (EVIDENCE / alias).write_text(stdout, encoding="utf-8")


def run(name: str, command: list[str], *, expected: int | None = None) -> int:
    proc = subprocess.run(command, text=True, capture_output=True, timeout=180)
    record(name, command, proc.stdout, proc.stderr, proc.returncode)
    if expected is not None and proc.returncode != expected:
        raise SystemExit(f"{name} failed: expected {expected}, got {proc.returncode}")
    return proc.returncode


def runtime_facts(name: str, expected_sha: str) -> dict[str, object]:
    output = EVIDENCE / f"{name}.stdout"
    status = int((EVIDENCE / f"{name}.status").read_text().strip())
    text = output.read_text(encoding="utf-8")
    digest = sha(output)
    lines = len(text.splitlines())
    if status != 0 or lines != 267 or digest != expected_sha:
        raise SystemExit(
            f"{name} facts mismatch: status={status}, lines={lines}, sha256={digest}"
        )
    return {
        "exit": status,
        "lines": lines,
        "stdout_sha256": digest,
    }


def main() -> None:
    cli_start = sha(CLI)
    if cli_start != EXPECTED_CLI_SHA:
        raise SystemExit(f"unexpected frozen CLI hash: {cli_start}")
    (EVIDENCE / "cli-sha-before.txt").write_text(f"{cli_start}  {CLI}\n", encoding="utf-8")

    with tempfile.TemporaryDirectory(prefix="jarde-narrow-field-stores-p3-") as temp:
        work = Path(temp)
        source_classes = work / "source-classes"
        patched_classes = work / "patched-classes"
        source_runner = work / "source-runner"
        patched_runner = work / "patched-runner"
        jarde = work / "jarde"
        jadx = work / "jadx"
        jadx_support = work / "jadx-support" / "defpackage"
        jadx_classes = work / "jadx-classes"
        for directory in (
            source_classes,
            patched_classes,
            source_runner,
            patched_runner,
            jarde,
            jadx,
            jadx_support,
            jadx_classes,
        ):
            directory.mkdir(parents=True, exist_ok=True)

        run(
            "source-javac",
            ["javac", "--release", "8", "-g:none", "-d", str(source_classes), str(SOURCE), str(EFFECTS)],
            expected=0,
        )
        source_class = source_classes / "NarrowFieldStores.class"
        source_effects = source_classes / "NarrowFieldStoreEffects.class"
        if sha(source_class) != EXPECTED_SOURCE_SHA:
            raise SystemExit(f"source class hash mismatch: {sha(source_class)}")
        if class_major(source_class) != 52:
            raise SystemExit(f"unexpected source class-file major: {class_major(source_class)}")
        shutil.copy2(source_class, EVIDENCE / "NarrowFieldStores.source.class")
        shutil.copy2(source_effects, EVIDENCE / "NarrowFieldStoreEffects.source.class")

        run(
            "patch",
            ["python3", str(PATCHER), str(source_class), str(patched_classes / "NarrowFieldStores.class")],
            expected=0,
        )
        patched_class = patched_classes / "NarrowFieldStores.class"
        if sha(patched_class) != EXPECTED_PATCHED_SHA:
            raise SystemExit(f"patched class hash mismatch: {sha(patched_class)}")
        if class_major(patched_class) != 52:
            raise SystemExit(f"unexpected patched class-file major: {class_major(patched_class)}")
        shutil.copy2(patched_class, EVIDENCE / "NarrowFieldStores.patched.class")
        raw_report = json.loads((patched_classes / "NarrowFieldStores.patch.json").read_text())
        raw_report["source"] = "NarrowFieldStores.source.class"
        raw_report["output"] = "NarrowFieldStores.patched.class"
        (EVIDENCE / "patch-report.json").write_text(json.dumps(raw_report, indent=2) + "\n", encoding="utf-8")
        if raw_report.get("method_count") != 19 or len(raw_report.get("methods", [])) != 19:
            raise SystemExit("patch report does not contain exactly 19 methods")
        if any(item.get("code_sha256") != item.get("code_sha256_after") for item in raw_report["methods"]):
            raise SystemExit("patch changed a method Code byte sequence")

        run("source-javap", ["javap", "-p", "-c", "-v", str(source_class)], expected=0)
        run("patched-javap", ["javap", "-p", "-c", "-v", str(patched_class)], expected=0)

        run(
            "source-runner-javac",
            ["javac", "--release", "8", "-g:none", "-cp", str(source_classes), "-d", str(source_runner), str(RUNNER)],
            expected=0,
        )
        run(
            "patched-runner-javac",
            [
                "javac", "--release", "8", "-g:none", "-cp", os.pathsep.join((str(patched_classes), str(source_classes))),
                "-d", str(patched_runner), str(RUNNER),
            ],
            expected=0,
        )
        run(
            "source-runtime",
            ["java", "-Xverify:all", "-cp", os.pathsep.join((str(source_runner), str(source_classes))), "NarrowFieldStoresRunner"],
            expected=0,
        )
        source_runtime = runtime_facts("source-runtime", EXPECTED_SOURCE_RUNTIME_SHA)
        run(
            "patched-runtime",
            ["java", "-Xverify:all", "-cp", os.pathsep.join((str(patched_runner), str(patched_classes), str(source_classes))), "NarrowFieldStoresRunner"],
            expected=0,
        )
        patched_runtime = runtime_facts("patched-runtime", EXPECTED_PATCHED_RUNTIME_SHA)

        cli_command = [
            str(CLI), "class-source", "--input", str(patched_class), "--class", "NarrowFieldStores",
            "--policy", "single-class", "--release", "8", "--format", "text", "--evidence", "all",
        ]
        cli = subprocess.run(cli_command, text=True, capture_output=True, timeout=180)
        record("jarde-cli", cli_command, cli.stdout, cli.stderr, cli.returncode)
        if cli.returncode != 0:
            raise SystemExit("fixed jarde CLI failed")
        (EVIDENCE / "jarde.java.txt").write_text(cli.stdout, encoding="utf-8")
        (EVIDENCE / "jarde-report.txt").write_text(cli.stderr, encoding="utf-8")
        quote_count = cli.stdout.count("@bytecode")
        if quote_count != 0:
            raise SystemExit(f"unexpected jarde quote count: {quote_count}")
        jarde_source = jarde / "NarrowFieldStores.java"
        jarde_source.write_text(cli.stdout, encoding="utf-8")
        run(
            "jarde-javac",
            ["javac", "--release", "8", "-g:none", "-d", str(jarde / "classes"), str(jarde_source), str(EFFECTS), str(RUNNER)],
            expected=0,
        )
        run(
            "jarde-runtime",
            ["java", "-Xverify:all", "-cp", str(jarde / "classes"), "NarrowFieldStoresRunner"],
            expected=0,
        )
        jarde_runtime = runtime_facts("jarde-runtime", sha(EVIDENCE / "jarde-runtime.stdout"))

        run("jadx", ["jadx", "--no-res", "-d", str(jadx), str(patched_class)], expected=0)
        generated = jadx / "sources" / "defpackage" / "NarrowFieldStores.java"
        if not generated.exists():
            raise SystemExit(f"JADX did not emit {generated}")
        shutil.copy2(generated, EVIDENCE / "jadx.java.txt")
        for source in (EFFECTS, RUNNER):
            target = jadx_support / source.name
            target.write_text("package defpackage;\n\n" + source.read_text(encoding="utf-8"), encoding="utf-8")
        jadx_javac_status = run(
            "jadx-javac",
            ["javac", "--release", "8", "-g:none", "-d", str(jadx_classes), str(generated), str(jadx_support / EFFECTS.name), str(jadx_support / RUNNER.name)],
        )
        jadx_runtime = None
        if jadx_javac_status == 0:
            run("jadx-runtime", ["java", "-Xverify:all", "-cp", str(jadx_classes), "defpackage.NarrowFieldStoresRunner"], expected=0)
            jadx_runtime = runtime_facts("jadx-runtime", sha(EVIDENCE / "jadx-runtime.stdout"))

    cli_end = sha(CLI)
    (EVIDENCE / "cli-sha-after.txt").write_text(f"{cli_end}  {CLI}\n", encoding="utf-8")
    if cli_end != cli_start:
        raise SystemExit(f"CLI changed during audit: {cli_end}")

    code_hashes = {
        f"{item['method']}{item['descriptor']}": item["code_sha256"]
        for item in raw_report["methods"]
    }
    summary = {
        "source_class_bytes": (EVIDENCE / "NarrowFieldStores.source.class").stat().st_size,
        "patched_class_bytes": (EVIDENCE / "NarrowFieldStores.patched.class").stat().st_size,
        "source_class_sha256": sha(EVIDENCE / "NarrowFieldStores.source.class"),
        "patched_class_sha256": sha(EVIDENCE / "NarrowFieldStores.patched.class"),
        "class_file_major": 52,
        "method_count": raw_report["method_count"],
        "field_count": raw_report["field_count"],
        "target_fields": {field["name"]: field["patched_descriptor"] for field in raw_report["fields"]},
        "code_attributes_byte_identical": all(
            item["code_sha256"] == item["code_sha256_after"] for item in raw_report["methods"]
        ),
        "code_sha256_by_method": code_hashes,
        "runtime": {"source": source_runtime, "patched": patched_runtime},
        "jarde": {"cli_exit": 0, "quote_count": quote_count, "javac_exit": 0, "runtime": jarde_runtime},
        "jadx": {"exit": 0, "javac_exit": jadx_javac_status, "runtime": jadx_runtime},
        "cli_sha256": cli_end,
    }
    if not summary["code_attributes_byte_identical"]:
        raise SystemExit("summary detected changed Code bytes")
    (EVIDENCE / "summary.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"summary": str(EVIDENCE / "summary.json"), **summary}, indent=2))


if __name__ == "__main__":
    main()
