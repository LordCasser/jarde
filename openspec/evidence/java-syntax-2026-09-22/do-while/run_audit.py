from __future__ import annotations

import difflib
import hashlib
import json
import re
import shutil
import subprocess
from pathlib import Path


OUT = Path(__file__).resolve().parent
WORK = Path("/tmp/jarde-do-while-audit")
CLI = Path("/tmp/jarde-cli-deferred-budget-908c")
EXPECTED_CLI_SHA256 = "908c472560d6146354e1fd679c595e884afadf5ec7051727b51e79a1f95c6570"


def run_capture(command: list[str], stdout_path: Path, stderr_path: Path) -> int:
    result = subprocess.run(command, capture_output=True, text=True, timeout=90)
    stdout_path.write_text(result.stdout)
    stderr_path.write_text(result.stderr)
    return result.returncode


def run_log(command: list[str], log_path: Path) -> int:
    result = subprocess.run(command, capture_output=True, text=True, timeout=90)
    log_path.write_text(result.stdout + result.stderr)
    return result.returncode


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def package_info(source: str) -> tuple[str, str]:
    match = re.search(r"^package\s+([A-Za-z_$][\w$]*(?:\.[A-Za-z_$][\w$]*)*)\s*;", source, re.MULTILINE)
    if not match:
        return "", ""
    return match.group(1), match.group(0) + "\n"


def candidate_compile_run(
    source: Path,
    runner_source: Path,
    work: Path,
    evidence: Path,
    prefix: str,
    runner_name: str,
    package_name: str = "",
    package_line: str = "",
) -> dict[str, object]:
    classes = work / "classes"
    classes.mkdir(parents=True, exist_ok=True)
    runner = work / f"{runner_name}.java"
    runner.write_text(package_line + runner_source.read_text())
    javac = run_log(
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
        evidence / f"{prefix}-javac.log",
    )
    result: dict[str, object] = {"javac": javac}
    stdout = evidence / f"{prefix}-run.txt"
    stderr = evidence / f"{prefix}-run.stderr.txt"
    if javac == 0:
        main = f"{package_name}.{runner_name}" if package_name else runner_name
        result["run"] = run_capture(
            ["java", "-Xverify:all", "-cp", str(classes), main], stdout, stderr
        )
    else:
        stdout.write_text("")
        stderr.write_text("")
        result["run"] = None
    if stdout.exists():
        result["output_sha256"] = sha256(stdout)
    return result


def save_diff(left: Path, right: Path, destination: Path) -> None:
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


def audit_case(
    class_name: str,
    runner_name: str,
    source: Path,
    runner_source: Path,
    evidence: Path,
    work: Path,
) -> dict[str, object]:
    evidence.mkdir(parents=True, exist_ok=True)
    work.mkdir(parents=True, exist_ok=True)
    original = work / "original"
    original.mkdir(parents=True, exist_ok=True)
    summary: dict[str, object] = {
        "class": class_name,
        "source": source.name,
        "runner": runner_source.name,
        "source_javac": run_log(
            ["javac", "--release", "8", "-g:none", "-d", str(original), str(source)],
            evidence / "source-javac.log",
        ),
    }
    class_file = original / f"{class_name}.class"
    if summary["source_javac"] != 0:
        return summary
    summary["class_bytes"] = class_file.stat().st_size
    summary["class_sha256"] = sha256(class_file)
    runner_classes = original
    runner = work / f"{runner_name}.java"
    runner.write_text(runner_source.read_text())
    summary["runner_javac"] = run_log(
        [
            "javac",
            "--release",
            "8",
            "-g:none",
            "-cp",
            str(runner_classes),
            "-d",
            str(runner_classes),
            str(runner),
        ],
        evidence / "runner-javac.log",
    )
    if summary["runner_javac"] == 0:
        summary["original_run"] = run_capture(
            ["java", "-Xverify:all", "-cp", str(original), runner_name],
            evidence / "original-run.txt",
            evidence / "original-run.stderr.txt",
        )
        summary["original_output_sha256"] = sha256(evidence / "original-run.txt")
    else:
        (evidence / "original-run.txt").write_text("")
        (evidence / "original-run.stderr.txt").write_text("")
    summary["javap"] = run_log(
        ["javap", "-c", "-v", "-p", str(class_file)], evidence / "original-javap.txt"
    )

    jadx_work = work / "jadx"
    summary["jadx_extract"] = run_log(
        ["jadx", "--no-res", "-d", str(jadx_work), str(class_file)], evidence / "jadx.log"
    )
    generated = next(jadx_work.rglob(f"{class_name}.java"), None)
    if generated is None:
        summary["jadx"] = None
    else:
        generated_text = generated.read_text()
        (evidence / "jadx.java.txt").write_text(generated_text)
        package_name, package_line = package_info(generated_text)
        summary["jadx_package"] = package_name
        summary["jadx"] = candidate_compile_run(
            generated,
            runner_source,
            work / "jadx-compiled",
            evidence,
            "jadx",
            runner_name,
            package_name,
            package_line,
        )
        if summary["original_run"] == 0 and summary["jadx"]["run"] == 0:
            save_diff(evidence / "original-run.txt", evidence / "jadx-run.txt", evidence / "jadx-diff.txt")

    frozen_sha = sha256(CLI)
    summary["cli_sha256"] = frozen_sha
    summary["cli_sha256_expected"] = EXPECTED_CLI_SHA256
    summary["cli_sha256_matches"] = frozen_sha == EXPECTED_CLI_SHA256
    (evidence / "cli-sha256.txt").write_text(f"{frozen_sha}  {CLI}\n")
    if not summary["cli_sha256_matches"]:
        raise RuntimeError(f"unexpected frozen CLI SHA-256: {frozen_sha}")

    frozen = evidence / "jarde.java.txt"
    report = evidence / "jarde-report.txt"
    result = subprocess.run(
        [
            str(CLI),
            "class-source",
            "--input",
            str(class_file),
            "--class",
            class_name,
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
    frozen.write_text(result.stdout)
    report.write_text(result.stderr)
    summary["jarde_class_source"] = result.returncode
    summary["jarde_quotes"] = result.stdout.count("@bytecode")
    if result.returncode == 0:
        package_name, package_line = package_info(result.stdout)
        summary["jarde_package"] = package_name
        frozen_source = work / "jarde-source" / f"{class_name}.java"
        frozen_source.parent.mkdir(parents=True, exist_ok=True)
        frozen_source.write_text(result.stdout)
        summary["jarde"] = candidate_compile_run(
            frozen_source,
            runner_source,
            work / "jarde-compiled",
            evidence,
            "jarde",
            runner_name,
            package_name,
            package_line,
        )
        if summary["original_run"] == 0 and summary["jarde"]["run"] == 0:
            save_diff(evidence / "original-run.txt", evidence / "jarde-run.txt", evidence / "jarde-diff.txt")
        else:
            (evidence / "jarde-diff.txt").write_text("runtime comparison skipped\n")
    else:
        summary["jarde"] = None
        (evidence / "jarde-javac.log").write_text("class-source failed; source was not compiled\n")
        (evidence / "jarde-run.txt").write_text("")
        (evidence / "jarde-run.stderr.txt").write_text("")
        (evidence / "jarde-diff.txt").write_text("class-source failed; source was not compiled\n")
    return summary


def main() -> None:
    if not CLI.exists():
        raise SystemExit(f"missing frozen CLI: {CLI}")
    summaries = {
        "positive": audit_case(
            "DoWhilePositive",
            "DoWhilePositiveRunner",
            OUT / "positive" / "DoWhilePositive.java",
            OUT / "positive" / "DoWhilePositiveRunner.java",
            OUT / "positive",
            WORK / "positive",
        ),
        "full": audit_case(
            "DoWhileAudit",
            "DoWhileRunner",
            OUT / "DoWhileAudit.java",
            OUT / "DoWhileRunner.java",
            OUT,
            WORK / "full",
        ),
        "minimal_core": audit_case(
            "DoWhileCore",
            "DoWhileCoreRunner",
            OUT / "core" / "DoWhileCore.java",
            OUT / "core" / "DoWhileCoreRunner.java",
            OUT / "core",
            WORK / "core",
        ),
    }
    (OUT / "summary.json").write_text(json.dumps(summaries, indent=2) + "\n")
    print(json.dumps(summaries, indent=2))


if __name__ == "__main__":
    main()
