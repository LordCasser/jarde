from __future__ import annotations

import difflib
import hashlib
import json
import shutil
import subprocess
from pathlib import Path


OUT = Path(__file__).resolve().parent
WORK = Path("/tmp/jarde-string-switch-audit")
CLI = Path("/tmp/jarde-cli-deferred-budget-908c")
CLASS = "StringSwitchProbe"
RUNNER = "StringSwitchRunner"


def run(command: list[str], log: Path, *, cwd: Path | None = None) -> int:
    result = subprocess.run(command, cwd=cwd, capture_output=True, text=True, timeout=120)
    log.write_text(result.stdout + result.stderr)
    return result.returncode


def package(source: str) -> str:
    return next((line + "\n" for line in source.splitlines() if line.startswith("package ")), "")


def compare(left: Path, right: Path, output: Path) -> None:
    output.write_text(
        "".join(
            difflib.unified_diff(
                left.read_text().splitlines(keepends=True),
                right.read_text().splitlines(keepends=True),
                fromfile=left.name,
                tofile=right.name,
            )
        )
    )


def compile_and_run(source: Path, target: str, result: dict[str, object]) -> None:
    work = WORK / target
    work.mkdir(parents=True, exist_ok=True)
    copy = work / f"{CLASS}.java"
    copy.write_text(source.read_text())
    runner = work / f"{RUNNER}.java"
    runner.write_text(package(copy.read_text()) + (OUT / f"{RUNNER}.java").read_text())
    classes = work / "classes"
    classes.mkdir(exist_ok=True)
    result[f"{target}_javac"] = run(
        ["javac", "--release", "8", "-g:none", "-d", str(classes), str(copy), str(runner)],
        OUT / f"{target}-javac.log",
    )
    if result[f"{target}_javac"] == 0:
        runner_name = (package(copy.read_text()).strip().removeprefix("package ").removesuffix(";") + "." if package(copy.read_text()) else "") + RUNNER
        result[f"{target}_run"] = run(
            ["java", "-Xverify:all", "-cp", str(classes), runner_name],
            OUT / f"{target}-run.txt",
        )
    else:
        result[f"{target}_run"] = None
        (OUT / f"{target}-run.txt").write_text("")


def main() -> None:
    WORK.mkdir(exist_ok=True)
    expected_cli_sha = "908c472560d6146354e1fd679c595e884afadf5ec7051727b51e79a1f95c6570"
    actual_cli_sha = hashlib.sha256(CLI.read_bytes()).hexdigest()
    if actual_cli_sha != expected_cli_sha:
        raise SystemExit(f"frozen CLI changed: {actual_cli_sha}")
    original_dir = WORK / "original"
    original_dir.mkdir(exist_ok=True)
    summary: dict[str, object] = {
        "javac_path": shutil.which("javac"),
        "jadx_path": shutil.which("jadx"),
        "cli": str(CLI),
        "cli_sha256": actual_cli_sha,
    }
    summary["source_javac"] = run(
        ["javac", "--release", "8", "-g:none", "-d", str(original_dir), str(OUT / f"{CLASS}.java")],
        OUT / "source-javac.log",
    )
    if summary["source_javac"] != 0:
        (OUT / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
        raise SystemExit("source did not compile")
    class_file = original_dir / f"{CLASS}.class"
    summary["class_bytes"] = class_file.stat().st_size
    summary["class_sha256"] = hashlib.sha256(class_file.read_bytes()).hexdigest()
    summary["javap"] = run(["javap", "-c", "-p", "-v", str(class_file)], OUT / "javap.txt")
    compile_and_run(OUT / f"{CLASS}.java", "original", summary)

    jadx_dir = WORK / "jadx"
    summary["jadx_extract"] = run(
        ["jadx", "--no-res", "-d", str(jadx_dir), str(class_file)], OUT / "jadx.log"
    )
    jadx_source = next(jadx_dir.rglob(f"{CLASS}.java"), None)
    if jadx_source is not None:
        (OUT / "jadx.java.txt").write_text(jadx_source.read_text())
        compile_and_run(jadx_source, "jadx", summary)
        compare(OUT / "original-run.txt", OUT / "jadx-run.txt", OUT / "jadx-diff.txt")

    generated = subprocess.run(
        [str(CLI), "class-source", "--input", str(class_file), "--class", CLASS,
         "--policy", "single-class", "--release", "8", "--format", "text"],
        capture_output=True, text=True, timeout=120,
    )
    summary["jarde_class_source"] = generated.returncode
    summary["jarde_quotes"] = generated.stdout.count("@bytecode")
    (OUT / "jarde.java.txt").write_text(generated.stdout)
    (OUT / "jarde-report.txt").write_text(generated.stderr)
    if generated.returncode == 0:
        compile_and_run(OUT / "jarde.java.txt", "jarde", summary)
        compare(OUT / "original-run.txt", OUT / "jarde-run.txt", OUT / "jarde-diff.txt")
    (OUT / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")


if __name__ == "__main__":
    main()
