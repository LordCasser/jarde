#!/usr/bin/env python3
"""Reproduce the int[] -> narrow-array method_info/opcode audit without Cargo."""
from __future__ import annotations

import hashlib
import os
import shutil
import subprocess
import tempfile
from pathlib import Path

EVIDENCE = Path(__file__).resolve().parent
CLI = Path("/tmp/jarde-cli-deferred-interim-948a")
EXPECTED_CLI_SHA = "948a6f9b7db009fbb89a792f83328c9ab63e42ac0cb62e691d97bec28c2c8ba0"
SOURCE = EVIDENCE / "NarrowArrayStores.java"
EFFECTS = EVIDENCE / "NarrowArrayStoreEffects.java"
RUNNER = EVIDENCE / "NarrowArrayStoresRunner.java"
PATCHER = EVIDENCE / "patch_array_stores.py"
TEXT_OUTPUTS = {
    "original-runtime", "patched-runtime", "jarde-runtime", "jadx-runtime",
    "original-javap", "patched-javap", "original-signatures", "patched-signatures",
}


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def record_command(name: str, command: list[str], stdout: str = "", stderr: str = "", status: int = 0) -> None:
    (EVIDENCE / f"{name}.command.txt").write_text(" ".join(command) + "\n")
    (EVIDENCE / f"{name}.stdout").write_text(stdout)
    if name in TEXT_OUTPUTS:
        (EVIDENCE / f"{name}.txt").write_text(stdout)
    (EVIDENCE / f"{name}.stderr").write_text(stderr)
    (EVIDENCE / f"{name}.status").write_text(f"{status}\n")


def run(name: str, command: list[str], cwd: Path | None = None) -> int:
    proc = subprocess.run(command, cwd=cwd, text=True, capture_output=True, timeout=45)
    record_command(name, command, proc.stdout, proc.stderr, proc.returncode)
    return proc.returncode


def main() -> None:
    cli_sha = sha(CLI)
    if cli_sha != EXPECTED_CLI_SHA:
        raise SystemExit(f"unexpected frozen CLI hash: {cli_sha}")
    (EVIDENCE / "cli-sha256-start.txt").write_text(f"{cli_sha}  {CLI}\n")
    with tempfile.TemporaryDirectory(prefix="jarde-narrow-array-stores-") as temp:
        work = Path(temp)
        original = work / "original"
        patched = work / "patched"
        original_run = work / "original-run"
        patched_run = work / "patched-run"
        jarde = work / "jarde"
        jadx = work / "jadx"
        jadx_support = work / "jadx-support" / "defpackage"
        jadx_classes = work / "jadx-classes"
        for directory in (original, patched, original_run, patched_run, jarde, jadx, jadx_support, jadx_classes):
            directory.mkdir(parents=True, exist_ok=True)

        source_javac = ["javac", "--release", "8", "-g:none", "-d", str(original), str(SOURCE), str(EFFECTS)]
        source_status = run("source-javac", source_javac)
        if source_status:
            raise SystemExit("source compilation failed")
        shutil.copy2(original / "NarrowArrayStores.class", EVIDENCE / "NarrowArrayStores.source.class")
        shutil.copy2(original / "NarrowArrayStoreEffects.class", EVIDENCE / "NarrowArrayStoreEffects.source.class")

        patch_status = run("patch", ["python3", str(PATCHER), str(original / "NarrowArrayStores.class"), str(patched / "NarrowArrayStores.class")])
        if patch_status:
            raise SystemExit("class patch failed")
        shutil.copy2(patched / "NarrowArrayStores.class", EVIDENCE / "NarrowArrayStores.patched.class")
        import json
        patch_report = json.loads((patched / "NarrowArrayStores.patch.json").read_text())
        patch_report["source"] = "NarrowArrayStores.source.class"
        patch_report["output"] = "NarrowArrayStores.patched.class"
        (EVIDENCE / "patch-report.detail.json").write_text(json.dumps(patch_report, indent=2) + "\n")
        (EVIDENCE / "patch-report.json").write_text(json.dumps(patch_report, indent=2) + "\n")
        with (EVIDENCE / "class-sha256.txt").open("w") as output:
            output.write(f"{sha(original / 'NarrowArrayStores.class')}  NarrowArrayStores.source.class\n")
            output.write(f"{sha(patched / 'NarrowArrayStores.class')}  NarrowArrayStores.patched.class\n")

        for label, classpath, destination in (
            ("original-runner-javac", original, original_run),
            ("patched-runner-javac", Path(os.pathsep.join((str(patched), str(original)))), patched_run),
        ):
            run("%s" % label, ["javac", "--release", "8", "-g:none", "-cp", str(classpath), "-d", str(destination), str(RUNNER)])
        run("original-runtime", ["java", "-Xverify:all", "-cp", os.pathsep.join((str(original_run), str(original))), "NarrowArrayStoresRunner", "original"])
        run("patched-runtime", ["java", "-Xverify:all", "-cp", os.pathsep.join((str(patched_run), str(patched), str(original))), "NarrowArrayStoresRunner"])

        run("original-javap", ["javap", "-p", "-c", "-v", str(original / "NarrowArrayStores.class")])
        run("patched-javap", ["javap", "-p", "-c", "-v", str(patched / "NarrowArrayStores.class")])
        run("original-signatures", ["javap", "-p", "-s", str(original / "NarrowArrayStores.class")])
        run("patched-signatures", ["javap", "-p", "-s", str(patched / "NarrowArrayStores.class")])

        jarde_source = jarde / "NarrowArrayStores.java"
        jarde_source.write_text(subprocess.run(
            [str(CLI), "class-source", "--input", str(patched / "NarrowArrayStores.class"), "--class", "NarrowArrayStores", "--policy", "single-class", "--release", "8", "--format", "text", "--evidence", "all"],
            text=True, capture_output=True, check=True,
        ).stdout)
        (EVIDENCE / "jarde.java.txt").write_text(jarde_source.read_text())
        report = subprocess.run(
            [str(CLI), "class-source", "--input", str(patched / "NarrowArrayStores.class"), "--class", "NarrowArrayStores", "--policy", "single-class", "--release", "8", "--format", "text", "--evidence", "all"],
            text=True, capture_output=True, check=True,
        )
        (EVIDENCE / "jarde-report.txt").write_text(report.stderr)
        run("jarde-javac", ["javac", "--release", "8", "-g:none", "-d", str(jarde / "classes"), str(jarde_source), str(EFFECTS), str(RUNNER)])
        if (EVIDENCE / "jarde-javac.status").read_text().strip() == "0":
            run("jarde-runtime", ["java", "-Xverify:all", "-cp", str(jarde / "classes"), "NarrowArrayStoresRunner"])

        run("jadx", ["jadx", "--no-res", "-d", str(jadx), str(patched / "NarrowArrayStores.class")])
        generated = jadx / "sources" / "defpackage" / "NarrowArrayStores.java"
        if generated.exists():
            shutil.copy2(generated, EVIDENCE / "jadx.java.txt")
            shutil.copy2(EFFECTS, jadx_support / EFFECTS.name)
            shutil.copy2(RUNNER, jadx_support / RUNNER.name)
            for source in (jadx_support / EFFECTS.name, jadx_support / RUNNER.name):
                source.write_text("package defpackage;\n\n" + source.read_text())
            run("jadx-javac", ["javac", "--release", "8", "-g:none", "-d", str(jadx_classes), str(generated), str(jadx_support / EFFECTS.name), str(jadx_support / RUNNER.name)])
            if (EVIDENCE / "jadx-javac.status").read_text().strip() == "0":
                run("jadx-runtime", ["java", "-Xverify:all", "-cp", str(jadx_classes), "defpackage.NarrowArrayStoresRunner"])

    assert sha(EVIDENCE / "NarrowArrayStores.patched.class") == "8bbb683045fdc2a85ee376383515920331fd15c53710176e447ba93f0d04004b"
    for stage in ["source-javac", "original-runner-javac", "patched-runner-javac", "original-runtime", "patched-runtime", "jarde-javac", "jarde-runtime"]:
        assert (EVIDENCE / f"{stage}.status").read_text().strip() == "0", stage
    assert len((EVIDENCE / "patched-runtime.txt").read_text().splitlines()) == 196
    end_sha = sha(CLI)
    (EVIDENCE / "cli-sha256-after.txt").write_text(f"{end_sha}  {CLI}\n")
    if end_sha != EXPECTED_CLI_SHA:
        raise SystemExit(f"CLI changed during audit: {end_sha}")


if __name__ == "__main__":
    main()
