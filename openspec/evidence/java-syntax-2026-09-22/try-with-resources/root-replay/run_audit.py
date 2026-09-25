from __future__ import annotations

import difflib
import hashlib
import json
import re
import shutil
import subprocess
import tempfile
from pathlib import Path


OUT = Path(__file__).resolve().parent
CLI = Path("/tmp/jarde-cli-class-literals-root")
EXPECTED_CLI_SHA256 = "25bf181ccf0d879351a16710818e27efba0df6b3851931a29eb98d9f41adb1e0"
SAMPLES = [
    ("core", "TwrAudit", "TwrAuditRunner"),
    ("multi-resource", "TwrMultiAudit", "TwrMultiAuditRunner"),
    ("null-resource", "NullResourceCore", "NullResourceRunner"),
]


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def package_name(source: str) -> str:
    match = re.search(r"^package\s+([^;]+);", source, re.MULTILINE)
    return match.group(1) if match else ""


def record_command(out: Path, prefix: str, args: list[str]) -> int:
    result = subprocess.run(args, capture_output=True, text=True, timeout=120)
    (out / f"{prefix}.stdout").write_text(result.stdout, encoding="utf-8")
    (out / f"{prefix}.stderr").write_text(result.stderr, encoding="utf-8")
    (out / f"{prefix}.status").write_text(f"{result.returncode}\n", encoding="utf-8")
    return result.returncode


def record_skipped(out: Path, prefix: str, why: str) -> None:
    (out / f"{prefix}.stdout").write_text("", encoding="utf-8")
    (out / f"{prefix}.stderr").write_text(f"not run: {why}\n", encoding="utf-8")
    (out / f"{prefix}.status").write_text("skipped\n", encoding="utf-8")


def compile_and_run(
    out: Path,
    work: Path,
    class_name: str,
    runner_name: str,
    variant: str,
    source_path: Path,
) -> dict[str, object]:
    variant_work = work / variant
    classes = variant_work / "classes"
    classes.mkdir(parents=True)
    source_text = source_path.read_text(encoding="utf-8")
    package = package_name(source_text)
    package_path = Path(*package.split(".")) if package else Path()
    qualified_runner = f"{package + '.' if package else ''}{runner_name}"
    compile_source = source_path
    if source_path.suffix != ".java":
        source_dir = variant_work / "source"
        source_dir.mkdir()
        compile_source = source_dir / f"{class_name}.java"
        shutil.copy2(source_path, compile_source)

    class_status = record_command(
        out,
        f"{variant}-javac",
        ["javac", "--release", "8", "-g:none", "-d", str(classes), str(compile_source)],
    )
    result: dict[str, object] = {"class_javac": class_status}
    class_file = classes / package_path / f"{class_name}.class"
    if class_status != 0 or not class_file.exists():
        record_skipped(out, f"{variant}-runner-javac", "whole-class javac failed")
        record_skipped(out, f"{variant}-runtime", "whole-class javac failed")
        return result

    result["class_bytes"] = class_file.stat().st_size
    result["class_sha256"] = sha256(class_file)
    shutil.copy2(class_file, out / f"{variant}.class")

    runner_text = (out / f"{runner_name}.java").read_text(encoding="utf-8")
    runner_work = variant_work / "runner-source"
    runner_work.mkdir()
    runner_source = runner_work / f"{runner_name}.java"
    if package:
        runner_text = f"package {package};\n\n{runner_text}"
    runner_source.write_text(runner_text, encoding="utf-8")
    runner_status = record_command(
        out,
        f"{variant}-runner-javac",
        [
            "javac",
            "--release",
            "8",
            "-g:none",
            "-cp",
            str(classes),
            "-d",
            str(classes),
            str(runner_source),
        ],
    )
    result["runner_javac"] = runner_status
    if runner_status != 0:
        record_skipped(out, f"{variant}-runtime", "runner javac failed")
        return result

    runner_class = classes / package_path / f"{runner_name}.class"
    shutil.copy2(runner_class, out / f"{variant}-{runner_name}.class")
    runtime_status = record_command(
        out,
        f"{variant}-runtime",
        ["java", "-Xverify:all", "-cp", str(classes), qualified_runner],
    )
    shutil.copy2(out / f"{variant}-runtime.stdout", out / f"{variant}-runtime.txt")
    result["runtime"] = runtime_status
    result["runtime_output_sha256"] = sha256(out / f"{variant}-runtime.stdout")

    javap_status = record_command(
        out,
        f"{variant}-javap",
        ["javap", "-v", "-c", "-p", str(class_file)],
    )
    shutil.copy2(out / f"{variant}-javap.stdout", out / f"{variant}-javap.txt")
    result["javap"] = javap_status
    return result


def compare_runtime(out: Path, variant: str, original: bytes) -> dict[str, object]:
    status = (out / f"{variant}-runtime.status").read_text(encoding="utf-8").strip()
    if status == "skipped":
        (out / f"{variant}-vs-original.diff").write_text(
            f"comparison skipped: {variant} did not run\n", encoding="utf-8"
        )
        return {"status": "skipped", "equal": None}
    candidate = (out / f"{variant}-runtime.stdout").read_bytes()
    original_lines = original.decode("utf-8", errors="replace").splitlines(keepends=True)
    candidate_lines = candidate.decode("utf-8", errors="replace").splitlines(keepends=True)
    equal = status == "0" and original.splitlines() == candidate.splitlines()
    diff = difflib.unified_diff(
        original_lines,
        candidate_lines,
        fromfile="original-runtime.stdout",
        tofile=f"{variant}-runtime.stdout",
    )
    (out / f"{variant}-vs-original.diff").write_text(
        "".join(diff) if not equal else "line-by-line output identical\n",
        encoding="utf-8",
    )
    return {
        "status": int(status),
        "equal": equal,
        "lines": candidate.decode("utf-8", errors="replace").splitlines(),
    }


def audit_one(work: Path, folder: str, class_name: str, runner_name: str) -> dict[str, object]:
    out = OUT / folder
    source_path = out / f"{class_name}.java"
    runner_path = out / f"{runner_name}.java"
    result: dict[str, object] = {
        "class_source_sha256": sha256(source_path),
        "runner_source_sha256": sha256(runner_path),
    }
    result["original"] = compile_and_run(out, work / folder, class_name, runner_name, "original", source_path)
    original_class = work / folder / "original" / "classes" / f"{class_name}.class"
    if not original_class.exists():
        result["blocked"] = "source class did not compile"
        return result

    result["original_class_sha256"] = sha256(original_class)
    javap_status = record_command(
        out,
        "original-javap",
        ["javap", "-v", "-c", "-p", str(original_class)],
    )
    shutil.copy2(out / "original-javap.stdout", out / "original-javap.txt")
    result["original_javap"] = javap_status

    generated = subprocess.run(
        [
            str(CLI),
            "class-source",
            "--input",
            str(original_class),
            "--class",
            class_name,
            "--policy",
            "single-class",
            "--release",
            "8",
            "--format",
            "text",
            "--evidence",
            "all",
        ],
        capture_output=True,
        text=True,
        timeout=120,
    )
    (out / "jarde.java.txt").write_text(generated.stdout, encoding="utf-8")
    (out / "jarde-report.txt").write_text(generated.stderr, encoding="utf-8")
    (out / "jarde-cli.status").write_text(f"{generated.returncode}\n", encoding="utf-8")
    result["jarde_cli"] = generated.returncode
    result["jarde_source_sha256"] = sha256(out / "jarde.java.txt")

    jadx_dir = work / folder / "jadx"
    jadx_status = record_command(
        out,
        "jadx",
        ["jadx", "--no-res", "-d", str(jadx_dir), str(original_class)],
    )
    result["jadx"] = jadx_status
    jadx_source = next(jadx_dir.rglob(f"{class_name}.java"), None) if jadx_status == 0 else None
    if jadx_source is not None:
        shutil.copy2(jadx_source, out / "jadx.java.txt")
        result["jadx_source_sha256"] = sha256(out / "jadx.java.txt")
        result["jadx_package"] = package_name(jadx_source.read_text(encoding="utf-8"))
    else:
        result["jadx_source"] = "missing"

    if generated.returncode == 0:
        result["jarde"] = compile_and_run(
            out, work / folder, class_name, runner_name, "jarde", out / "jarde.java.txt"
        )
    else:
        record_skipped(out, "jarde-javac", "frozen CLI produced no class-source result")
        record_skipped(out, "jarde-runner-javac", "frozen CLI produced no class-source result")
        record_skipped(out, "jarde-runtime", "frozen CLI produced no class-source result")
        result["jarde"] = {"class_javac": "skipped"}

    if jadx_source is not None:
        result["jadx_decompiled"] = compile_and_run(
            out, work / folder, class_name, runner_name, "jadx", out / "jadx.java.txt"
        )
    else:
        record_skipped(out, "jadx-javac", "JADX produced no class source")
        record_skipped(out, "jadx-runner-javac", "JADX produced no class source")
        record_skipped(out, "jadx-runtime", "JADX produced no class source")
        result["jadx_decompiled"] = {"class_javac": "skipped"}

    original_status = (out / "original-runtime.status").read_text(encoding="utf-8").strip()
    original_output = (out / "original-runtime.stdout").read_bytes() if original_status == "0" else b""
    result["original_runtime"] = {
        "status": original_status,
        "lines": original_output.decode("utf-8", errors="replace").splitlines(),
    }
    result["jarde_comparison"] = compare_runtime(out, "jarde", original_output)
    result["jadx_comparison"] = compare_runtime(out, "jadx", original_output)
    result["source_lines"] = {
        "original": len(source_path.read_text(encoding="utf-8").splitlines()),
        "jarde": len((out / "jarde.java.txt").read_text(encoding="utf-8").splitlines()),
        "jadx": len((out / "jadx.java.txt").read_text(encoding="utf-8").splitlines())
        if (out / "jadx.java.txt").exists()
        else None,
    }
    (out / "audit.json").write_text(json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return result


def main() -> None:
    cli_before = sha256(CLI)
    (OUT / "cli-sha256-before.txt").write_text(f"{cli_before}  {CLI}\n", encoding="utf-8")
    if cli_before != EXPECTED_CLI_SHA256:
        raise SystemExit(f"frozen CLI hash mismatch before run: {cli_before}")

    version_parts = []
    for name, args in [
        ("java-version", ["java", "-version"]),
        ("javac-version", ["javac", "-version"]),
        ("javap-version", ["javap", "-version"]),
        ("jadx-version", ["jadx", "--version"]),
        ("cli-version", [str(CLI), "--version"]),
    ]:
        result = subprocess.run(args, capture_output=True, text=True, timeout=30)
        (OUT / f"{name}.stdout").write_text(result.stdout, encoding="utf-8")
        (OUT / f"{name}.stderr").write_text(result.stderr, encoding="utf-8")
        (OUT / f"{name}.status").write_text(f"{result.returncode}\n", encoding="utf-8")
        version_parts.append(f"[{name}]\n{result.stdout}{result.stderr}")
    (OUT / "tool-versions.txt").write_text("\n".join(version_parts), encoding="utf-8")

    results = {}
    with tempfile.TemporaryDirectory(prefix="jarde-twr-audit-") as temporary:
        work = Path(temporary)
        for folder, class_name, runner_name in SAMPLES:
            results[folder] = audit_one(work, folder, class_name, runner_name)

    cli_after = sha256(CLI)
    (OUT / "cli-sha256-after.txt").write_text(f"{cli_after}  {CLI}\n", encoding="utf-8")
    results["frozen_cli"] = {
        "path": str(CLI),
        "expected_sha256": EXPECTED_CLI_SHA256,
        "before_sha256": cli_before,
        "after_sha256": cli_after,
        "unchanged": cli_before == cli_after == EXPECTED_CLI_SHA256,
    }
    (OUT / "audit.json").write_text(json.dumps(results, indent=2, sort_keys=True) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
