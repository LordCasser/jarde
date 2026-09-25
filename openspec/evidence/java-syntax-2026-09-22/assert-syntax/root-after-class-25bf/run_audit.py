from __future__ import annotations

import difflib
import hashlib
import json
import shutil
import subprocess
from pathlib import Path


OUT = Path(__file__).resolve().parent
WORK = Path("/tmp/jarde-assert-syntax-audit-root-25bf")
CLI = Path("/tmp/jarde-cli-class-literals-root")
EXPECTED_CLI_SHA = "25bf181ccf0d879351a16710818e27efba0df6b3851931a29eb98d9f41adb1e0"
CLASS = "AssertProbe"
RUNNER = "AssertProbeRunner"
GENERATED = [
    "source-javac.log", "javap.txt", "jadx.log", "jarde-report.txt",
    "jarde.java.txt", "summary.json",
    "AssertProbe.class",
    "original-javac.log", "original-ea.txt", "original-da.txt",
    "jadx.java.txt", "jadx-javac.log", "jadx-ea.txt", "jadx-da.txt",
    "jadx-ea.diff", "jadx-da.diff",
    "jarde-javac.log", "jarde-ea.txt", "jarde-da.txt",
    "jarde-ea.diff", "jarde-da.diff",
]


def run(command: list[str], log: Path, *, cwd: Path | None = None) -> int:
    result = subprocess.run(command, cwd=cwd, capture_output=True, text=True, timeout=120)
    log.write_text(result.stdout + result.stderr)
    return result.returncode


def version(command: list[str]) -> str:
    result = subprocess.run(command, capture_output=True, text=True, timeout=30)
    return (result.stdout + result.stderr).strip()


def compare(left: Path, right: Path, output: Path) -> bool:
    left_lines = left.read_text().splitlines(keepends=True)
    right_lines = right.read_text().splitlines(keepends=True)
    output.write_text("".join(difflib.unified_diff(
        left_lines, right_lines, fromfile=left.name, tofile=right.name
    )))
    return left_lines == right_lines


def package_prefix(source: str) -> str:
    for line in source.splitlines():
        if line.startswith("package "):
            return line + "\n"
    return ""


def compile_and_run(source: Path, target: str, summary: dict[str, object]) -> None:
    work = WORK / target
    sources = work / "src"
    classes = work / "classes"
    sources.mkdir(parents=True, exist_ok=True)
    classes.mkdir(parents=True, exist_ok=True)
    probe_copy = sources / f"{CLASS}.java"
    probe_text = source.read_text()
    probe_copy.write_text(probe_text)
    runner_copy = sources / f"{RUNNER}.java"
    runner_copy.write_text(package_prefix(probe_text) + (OUT / f"{RUNNER}.java").read_text())

    javac_log = OUT / f"{target}-javac.log"
    compile_status = run(
        ["javac", "--release", "8", "-g:none", "-d", str(classes),
         str(probe_copy), str(runner_copy)],
        javac_log,
    )
    summary[f"{target}_javac"] = compile_status
    if compile_status != 0:
        for flag in ("ea", "da"):
            (OUT / f"{target}-{flag}.txt").write_text("")
            summary[f"{target}_{flag}_run"] = None
        summary[f"{target}_runtime_skip_reason"] = "javac_failed"
        return

    package = package_prefix(probe_text).strip()
    package_name = package[len("package "):].removesuffix(";") if package else ""
    runner_name = f"{package_name}.{RUNNER}" if package_name else RUNNER
    for flag, jvm_flag in (("ea", "-ea"), ("da", "-da")):
        summary[f"{target}_{flag}_run"] = run(
            ["java", "-Xverify:all", jvm_flag, "-cp", str(classes), runner_name],
            OUT / f"{target}-{flag}.txt",
        )


def main() -> None:
    for name in GENERATED:
        path = OUT / name
        if path.exists():
            path.unlink()
    if WORK.exists():
        shutil.rmtree(WORK)
    WORK.mkdir(parents=True)

    actual_cli_sha = hashlib.sha256(CLI.read_bytes()).hexdigest()
    if actual_cli_sha != EXPECTED_CLI_SHA:
        raise SystemExit(f"frozen CLI SHA mismatch: {actual_cli_sha}")

    summary: dict[str, object] = {
        "cli": str(CLI),
        "cli_sha256": actual_cli_sha,
        "javac_path": shutil.which("javac"),
        "javac_version": version(["javac", "-version"]),
        "java_path": shutil.which("java"),
        "java_version": version(["java", "-version"]),
        "jadx_path": shutil.which("jadx"),
        "jadx_version": version(["jadx", "--version"]),
        "compile_release": 8,
        "class_source_policy": "single-class",
        "probe_sha256": hashlib.sha256((OUT / f"{CLASS}.java").read_bytes()).hexdigest(),
        "runner_sha256": hashlib.sha256((OUT / f"{RUNNER}.java").read_bytes()).hexdigest(),
    }

    source = OUT / f"{CLASS}.java"
    original_dir = WORK / "original-input"
    original_dir.mkdir()
    source_status = run(
        ["javac", "--release", "8", "-g:none", "-d", str(original_dir), str(source)],
        OUT / "source-javac.log",
    )
    summary["source_javac"] = source_status
    if source_status != 0:
        (OUT / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
        raise SystemExit("source probe did not compile")

    class_file = original_dir / f"{CLASS}.class"
    shutil.copy2(class_file, OUT / f"{CLASS}.class")
    class_bytes = class_file.read_bytes()
    summary["class_bytes"] = len(class_bytes)
    summary["class_sha256"] = hashlib.sha256(class_bytes).hexdigest()
    javap_status = run(
        ["javap", "-c", "-p", "-v", str(class_file)], OUT / "javap.txt"
    )
    javap_text = (OUT / "javap.txt").read_text()
    assertion_shape = {
        "javap_succeeded": javap_status == 0,
        "assertions_disabled_field": "$assertionsDisabled" in javap_text,
        "desired_assertion_status_call": "desiredAssertionStatus" in javap_text,
    }
    summary["javap"] = javap_status
    summary["assertion_shape"] = assertion_shape
    if not all(assertion_shape.values()):
        (OUT / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
        raise SystemExit("javap did not show the expected javac assertion initialization shape")

    compile_and_run(source, "original", summary)

    jadx_dir = WORK / "jadx"
    jadx_status = run(
        ["jadx", "--no-res", "-d", str(jadx_dir), str(class_file)], OUT / "jadx.log"
    )
    summary["jadx_extract"] = jadx_status
    jadx_source = next(jadx_dir.rglob(f"{CLASS}.java"), None) if jadx_dir.exists() else None
    if jadx_source is not None:
        shutil.copy2(jadx_source, OUT / "jadx.java.txt")
        compile_and_run(OUT / "jadx.java.txt", "jadx", summary)
        if summary.get("original_ea_run") == 0 and summary.get("jadx_ea_run") == 0:
            summary["jadx_ea_equal"] = compare(
                OUT / "original-ea.txt", OUT / "jadx-ea.txt", OUT / "jadx-ea.diff"
            )
        if summary.get("original_da_run") == 0 and summary.get("jadx_da_run") == 0:
            summary["jadx_da_equal"] = compare(
                OUT / "original-da.txt", OUT / "jadx-da.txt", OUT / "jadx-da.diff"
            )
    else:
        summary["jadx_source_found"] = False
        summary["jadx_javac"] = None
        summary["jadx_ea_run"] = None
        summary["jadx_da_run"] = None
        summary["jadx_runtime_skip_reason"] = "no_jadx_source"

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
        if summary.get("original_ea_run") == 0 and summary.get("jarde_ea_run") == 0:
            summary["jarde_ea_equal"] = compare(
                OUT / "original-ea.txt", OUT / "jarde-ea.txt", OUT / "jarde-ea.diff"
            )
        if summary.get("original_da_run") == 0 and summary.get("jarde_da_run") == 0:
            summary["jarde_da_equal"] = compare(
                OUT / "original-da.txt", OUT / "jarde-da.txt", OUT / "jarde-da.diff"
            )
    else:
        summary["jarde_javac"] = None
        summary["jarde_ea_run"] = None
        summary["jarde_da_run"] = None
        summary["jarde_runtime_skip_reason"] = "class_source_failed"

    (OUT / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
