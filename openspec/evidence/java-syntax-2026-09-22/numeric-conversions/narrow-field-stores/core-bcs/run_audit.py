#!/usr/bin/env python3
"""Reproduce the descriptor/Fieldref narrow-field-store audit without Cargo."""
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
SOURCE = EVIDENCE / "NarrowFieldStores.java"
EFFECTS = EVIDENCE / "NarrowFieldStoreEffects.java"
RUNNER = EVIDENCE / "NarrowFieldStoresRunner.java"
PATCHER = EVIDENCE / "patch_field_stores.py"
TEXT_OUTPUTS = {
    "original-runtime", "patched-runtime", "jarde-runtime", "jadx-runtime",
    "original-javap", "patched-javap", "original-signatures", "patched-signatures",
}


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def record(name: str, command: list[str], stdout: str, stderr: str, status: int) -> None:
    (EVIDENCE / f"{name}.command.txt").write_text(" ".join(command) + "\n")
    (EVIDENCE / f"{name}.stdout").write_text(stdout)
    if name in TEXT_OUTPUTS:
        (EVIDENCE / f"{name}.txt").write_text(stdout)
    (EVIDENCE / f"{name}.stderr").write_text(stderr)
    (EVIDENCE / f"{name}.status").write_text(f"{status}\n")


def run(name: str, command: list[str]) -> int:
    proc = subprocess.run(command, text=True, capture_output=True)
    record(name, command, proc.stdout, proc.stderr, proc.returncode)
    return proc.returncode


def main() -> None:
    start_sha = sha(CLI)
    if start_sha != EXPECTED_CLI_SHA:
        raise SystemExit(f"unexpected frozen CLI hash: {start_sha}")
    (EVIDENCE / "cli-sha256-start.txt").write_text(f"{start_sha}  {CLI}\n")
    with tempfile.TemporaryDirectory(prefix="jarde-narrow-field-stores-") as temp:
        work = Path(temp)
        original = work / "original"
        patched = work / "patched"
        original_run = work / "run-original"
        patched_run = work / "run-patched"
        jarde = work / "jarde"
        jadx = work / "jadx"
        jadx_support = work / "jadx-support" / "defpackage"
        jadx_classes = work / "jadx-classes"
        for directory in (original, patched, original_run, patched_run, jarde, jadx, jadx_support, jadx_classes):
            directory.mkdir(parents=True, exist_ok=True)

        if run("source-javac", ["javac", "--release", "8", "-g:none", "-d", str(original), str(SOURCE), str(EFFECTS)]):
            raise SystemExit("source compilation failed")
        shutil.copy2(original / "NarrowFieldStores.class", EVIDENCE / "NarrowFieldStores.source.class")
        shutil.copy2(original / "NarrowFieldStoreEffects.class", EVIDENCE / "NarrowFieldStoreEffects.source.class")

        if run("patch", ["python3", str(PATCHER), str(original / "NarrowFieldStores.class"), str(patched / "NarrowFieldStores.class")]):
            raise SystemExit("field patch failed")
        shutil.copy2(patched / "NarrowFieldStores.class", EVIDENCE / "NarrowFieldStores.patched.class")
        patch_report = json.loads((patched / "NarrowFieldStores.patch.json").read_text())
        patch_report["source"] = "NarrowFieldStores.source.class"
        patch_report["output"] = "NarrowFieldStores.patched.class"
        (EVIDENCE / "patch-report.detail.json").write_text(json.dumps(patch_report, indent=2) + "\n")
        (EVIDENCE / "patch-report.json").write_text(json.dumps(patch_report, indent=2) + "\n")
        (EVIDENCE / "class-sha256.txt").write_text(
            f"{sha(original / 'NarrowFieldStores.class')}  NarrowFieldStores.source.class\n"
            f"{sha(patched / 'NarrowFieldStores.class')}  NarrowFieldStores.patched.class\n"
        )

        run("original-runner-javac", ["javac", "--release", "8", "-g:none", "-cp", str(original), "-d", str(original_run), str(RUNNER)])
        run("patched-runner-javac", ["javac", "--release", "8", "-g:none", "-cp", os.pathsep.join((str(patched), str(original))), "-d", str(patched_run), str(RUNNER)])
        run("original-runtime", ["java", "-Xverify:all", "-cp", os.pathsep.join((str(original_run), str(original))), "NarrowFieldStoresRunner", "original"])
        run("patched-runtime", ["java", "-Xverify:all", "-cp", os.pathsep.join((str(patched_run), str(patched), str(original))), "NarrowFieldStoresRunner"])
        run("original-javap", ["javap", "-p", "-c", "-v", str(original / "NarrowFieldStores.class")])
        run("patched-javap", ["javap", "-p", "-c", "-v", str(patched / "NarrowFieldStores.class")])
        run("original-signatures", ["javap", "-p", "-s", str(original / "NarrowFieldStores.class")])
        run("patched-signatures", ["javap", "-p", "-s", str(patched / "NarrowFieldStores.class")])

        cli_command = [str(CLI), "class-source", "--input", str(patched / "NarrowFieldStores.class"), "--class", "NarrowFieldStores", "--policy", "single-class", "--release", "8", "--format", "text", "--evidence", "all"]
        cli = subprocess.run(cli_command, text=True, capture_output=True)
        record("jarde-cli", cli_command, cli.stdout, cli.stderr, cli.returncode)
        if cli.returncode:
            raise SystemExit("jarde CLI failed")
        jarde_source = jarde / "NarrowFieldStores.java"
        jarde_source.write_text(cli.stdout)
        (EVIDENCE / "jarde.java.txt").write_text(cli.stdout)
        (EVIDENCE / "jarde-report.txt").write_text(cli.stderr)
        run("jarde-javac", ["javac", "--release", "8", "-g:none", "-d", str(jarde / "classes"), str(jarde_source), str(EFFECTS), str(RUNNER)])
        if (EVIDENCE / "jarde-javac.status").read_text().strip() == "0":
            run("jarde-runtime", ["java", "-Xverify:all", "-cp", str(jarde / "classes"), "NarrowFieldStoresRunner"])

        run("jadx", ["jadx", "--no-res", "-d", str(jadx), str(patched / "NarrowFieldStores.class")])
        generated = jadx / "sources" / "defpackage" / "NarrowFieldStores.java"
        if generated.exists():
            shutil.copy2(generated, EVIDENCE / "jadx.java.txt")
            for source in (EFFECTS, RUNNER):
                target = jadx_support / source.name
                shutil.copy2(source, target)
                target.write_text("package defpackage;\n\n" + target.read_text())
            run("jadx-javac", ["javac", "--release", "8", "-g:none", "-d", str(jadx_classes), str(generated), str(jadx_support / EFFECTS.name), str(jadx_support / RUNNER.name)])
            if (EVIDENCE / "jadx-javac.status").read_text().strip() == "0":
                run("jadx-runtime", ["java", "-Xverify:all", "-cp", str(jadx_classes), "defpackage.NarrowFieldStoresRunner"])

    end_sha = sha(CLI)
    (EVIDENCE / "cli-sha256-after.txt").write_text(f"{end_sha}  {CLI}\n")
    if end_sha != EXPECTED_CLI_SHA:
        raise SystemExit(f"CLI changed during audit: {end_sha}")


if __name__ == "__main__":
    main()
