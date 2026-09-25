from __future__ import annotations

import difflib
import hashlib
import json
import shutil
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[4]
OUT = Path(__file__).resolve().parent
WORK = Path("/tmp/jarde-ternary-values-audit")
CLI = Path("/tmp/jarde-cli-deferred-accepted-7747")
CLASS = "TernaryValues"
RUNNER = "TernaryRunner"


def execute(command: list[str], stdout_path: Path, stderr_path: Path) -> int:
    result = subprocess.run(command, capture_output=True, text=True, timeout=90)
    stdout_path.write_text(result.stdout)
    stderr_path.write_text(result.stderr)
    return result.returncode


def execute_combined(command: list[str], log_path: Path) -> int:
    result = subprocess.run(command, capture_output=True, text=True, timeout=90)
    log_path.write_text(result.stdout + result.stderr)
    return result.returncode


def diff(left: Path, right: Path, destination: Path) -> None:
    if not right.exists():
        destination.write_text(f"execution not run: missing {right.name}\n")
        return
    destination.write_text(
        "".join(
            difflib.unified_diff(
                left.read_text().splitlines(keepends=True),
                right.read_text().splitlines(keepends=True),
                fromfile=left.name,
                tofile=right.name,
            )
        )
    )


def package_line(source: str) -> str:
    for line in source.splitlines():
        if line.startswith("package "):
            return line + "\n"
    return ""


def compile_and_run(
    source: Path,
    runner_source: Path,
    work: Path,
    prefix: str,
    runner_name: str,
    package: str = "",
    output_dir: Path = OUT,
) -> dict[str, object]:
    output_dir.mkdir(parents=True, exist_ok=True)
    classes = work / "classes"
    classes.mkdir(parents=True, exist_ok=True)
    runner = work / f"{runner_name}.java"
    runner.write_text(package + runner_source.read_text())
    javac = execute_combined(
        [
            "javac",
            "--release",
            "8",
            "-g:none",
            "-d",
            str(classes),
            str(source),
            str(runner),
        ],
        output_dir / f"{prefix}-javac.log",
    )
    result: dict[str, object] = {"javac": javac}
    run_stdout = output_dir / f"{prefix}-run.txt"
    run_stderr = output_dir / f"{prefix}-run.stderr.txt"
    if javac == 0:
        qualified_runner = f"{package.split(';')[0].split()[-1]}.{runner_name}" if package else runner_name
        result["run"] = execute(
            ["java", "-Xverify:all", "-cp", str(classes), qualified_runner],
            run_stdout,
            run_stderr,
        )
    else:
        run_stdout.write_text("")
        run_stderr.write_text("")
        result["run"] = None
    return result


def main() -> None:
    WORK.mkdir(parents=True, exist_ok=True)
    source = OUT / f"{CLASS}.java"
    runner = OUT / f"{RUNNER}.java"
    original = WORK / "original"
    original.mkdir(parents=True, exist_ok=True)
    summary: dict[str, object] = {
        "class": CLASS,
        "runner": RUNNER,
        "javac": shutil.which("javac"),
        "java": shutil.which("java"),
        "jadx": shutil.which("jadx"),
        "cli": str(CLI),
        "cli_sha256": hashlib.sha256(CLI.read_bytes()).hexdigest(),
        "source_javac": execute_combined(
            ["javac", "--release", "8", "-g:none", "-d", str(original), str(source)],
            OUT / "source-javac.log",
        ),
    }
    class_file = original / f"{CLASS}.class"
    if summary["source_javac"] != 0:
        (OUT / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
        raise SystemExit(1)

    summary["class_bytes"] = class_file.stat().st_size
    summary["class_sha256"] = hashlib.sha256(class_file.read_bytes()).hexdigest()
    summary["original_javap"] = execute_combined(
        ["javap", "-c", "-v", "-p", str(class_file)], OUT / "original-javap.txt"
    )
    summary["original"] = compile_and_run(source, runner, original, "original", RUNNER)
    summary["original_output_sha256"] = hashlib.sha256(
        (OUT / "original-run.txt").read_bytes()
    ).hexdigest()

    jadx_work = WORK / "jadx"
    summary["jadx_extract"] = execute_combined(
        ["jadx", "--no-res", "-d", str(jadx_work), str(class_file)], OUT / "jadx.log"
    )
    generated = next(jadx_work.rglob(f"{CLASS}.java"), None)
    if generated is None:
        summary["jadx_source"] = None
    else:
        generated_text = generated.read_text()
        (OUT / "jadx.java.txt").write_text(generated_text)
        package = package_line(generated_text)
        summary["jadx_package"] = package.strip()
        summary["jadx"] = compile_and_run(
            generated,
            runner,
            WORK / "jadx-compiled",
            "jadx",
            RUNNER,
            package,
        )
        summary["jadx_output_sha256"] = hashlib.sha256(
            (OUT / "jadx-run.txt").read_bytes()
        ).hexdigest()
        diff(OUT / "original-run.txt", OUT / "jadx-run.txt", OUT / "jadx-diff.txt")

    frozen = OUT / "frozen.java.txt"
    frozen_report = OUT / "frozen-report.txt"
    frozen_result = subprocess.run(
        [
            str(CLI),
            "class-source",
            "--input",
            str(class_file),
            "--class",
            CLASS,
            "--policy",
            "single-class",
            "--release",
            "8",
            "--format",
            "text",
        ],
        capture_output=True,
        text=True,
        timeout=90,
    )
    frozen.write_text(frozen_result.stdout)
    frozen_report.write_text(frozen_result.stderr)
    summary["frozen_class_source"] = frozen_result.returncode
    summary["frozen_quotes"] = frozen_result.stdout.count("@bytecode")
    if frozen_result.returncode == 0:
        package = package_line(frozen_result.stdout)
        summary["frozen_package"] = package.strip()
        frozen_source = WORK / "frozen-source" / f"{CLASS}.java"
        frozen_source.parent.mkdir(parents=True, exist_ok=True)
        frozen_source.write_text(frozen_result.stdout)
        summary["frozen"] = compile_and_run(
            frozen_source,
            runner,
            WORK / "frozen",
            "frozen",
            RUNNER,
            package,
        )
        diff(OUT / "original-run.txt", OUT / "frozen-run.txt", OUT / "frozen-diff.txt")
    else:
        (OUT / "frozen-javac.log").write_text("class-source failed; source was not compiled\n")
        (OUT / "frozen-run.txt").write_text("")
        (OUT / "frozen-run.stderr.txt").write_text("")
        (OUT / "frozen-diff.txt").write_text("class-source failed; source was not compiled\n")

    core = OUT / "core"
    core_work = WORK / "core"
    core_source = core / "TernaryCore.java"
    core_runner = core / "TernaryCoreRunner.java"
    core_original = core_work / "original"
    core_original.mkdir(parents=True, exist_ok=True)
    core_summary: dict[str, object] = {
        "source_javac": execute_combined(
            ["javac", "--release", "8", "-g:none", "-d", str(core_original), str(core_source)],
            core / "source-javac.log",
        )
    }
    core_class = core_original / "TernaryCore.class"
    if core_summary["source_javac"] == 0:
        core_summary["class_bytes"] = core_class.stat().st_size
        core_summary["class_sha256"] = hashlib.sha256(core_class.read_bytes()).hexdigest()
        core_summary["runner_javac"] = execute_combined(
            [
                "javac",
                "--release",
                "8",
                "-g:none",
                "-cp",
                str(core_original),
                "-d",
                str(core_original),
                str(core_runner),
            ],
            core / "runner-javac.log",
        )
        if core_summary["runner_javac"] == 0:
            core_summary["original_run"] = execute(
                ["java", "-Xverify:all", "-cp", str(core_original), "TernaryCoreRunner"],
                core / "original-run.txt",
                core / "original-run.stderr.txt",
            )
            core_summary["original_output_sha256"] = hashlib.sha256(
                (core / "original-run.txt").read_bytes()
            ).hexdigest()
        else:
            (core / "original-run.txt").write_text("")
            (core / "original-run.stderr.txt").write_text("")
        core_summary["javap"] = execute_combined(
            ["javap", "-c", "-v", "-p", str(core_class)], core / "original-javap.txt"
        )
        core_jadx_work = core_work / "jadx"
        core_summary["jadx_extract"] = execute_combined(
            ["jadx", "--no-res", "-d", str(core_jadx_work), str(core_class)], core / "jadx.log"
        )
        core_generated = next(core_jadx_work.rglob("TernaryCore.java"), None)
        if core_generated is None:
            core_summary["jadx"] = None
        else:
            core_generated_text = core_generated.read_text()
            (core / "jadx.java.txt").write_text(core_generated_text)
            core_summary["jadx_package"] = package_line(core_generated_text).strip()
            core_summary["jadx"] = compile_and_run(
                core_generated,
                core_runner,
                core_work / "jadx-compiled",
                "jadx",
                "TernaryCoreRunner",
                package_line(core_generated_text),
                core,
            )
            core_summary["jadx_output_sha256"] = hashlib.sha256(
                (core / "jadx-run.txt").read_bytes()
            ).hexdigest()
            diff(core / "original-run.txt", core / "jadx-run.txt", core / "jadx-diff.txt")
        core_frozen = core / "frozen.java.txt"
        core_report = core / "frozen-report.txt"
        core_result = subprocess.run(
            [
                str(CLI),
                "class-source",
                "--input",
                str(core_class),
                "--class",
                "TernaryCore",
                "--policy",
                "single-class",
                "--release",
                "8",
                "--format",
                "text",
            ],
            capture_output=True,
            text=True,
            timeout=90,
        )
        core_frozen.write_text(core_result.stdout)
        core_report.write_text(core_result.stderr)
        core_summary["frozen_class_source"] = core_result.returncode
        if core_result.returncode == 0:
            core_frozen_source = core_work / "frozen-source" / "TernaryCore.java"
            core_frozen_source.parent.mkdir(parents=True, exist_ok=True)
            core_frozen_source.write_text(core_result.stdout)
            core_summary["frozen"] = compile_and_run(
                core_frozen_source,
                core_runner,
                core_work / "frozen",
                "frozen",
                "TernaryCoreRunner",
                package_line(core_result.stdout),
                core,
            )
            diff(core / "original-run.txt", core / "frozen-run.txt", core / "frozen-diff.txt")
        else:
            core_summary["frozen"] = None
            (core / "frozen-javac.log").write_text("class-source failed; source was not compiled\n")
            (core / "frozen-run.txt").write_text("")
            (core / "frozen-run.stderr.txt").write_text("")
            (core / "frozen-diff.txt").write_text("class-source failed; source was not compiled\n")
    summary["minimal_core"] = core_summary

    (OUT / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
