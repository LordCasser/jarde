from __future__ import annotations

import hashlib
import json
import re
import shutil
import subprocess
import tempfile
from pathlib import Path


OUT = Path(__file__).resolve().parent
CLI = Path("/tmp/jarde-cli-boolean-root")
EXPECTED_CLI_SHA256 = "8cd1f767ba8cde9928bacedc29263d44be8e5e61b46d3b6f00456fac1c7c77cd"
CASES = {
    "int-array": ("IntArrayForeach", "IntArrayForeachRunner"),
    "object-array": ("ObjectArrayForeach", "ObjectArrayForeachRunner"),
    "string-iterable": ("StringIterableForeach", "StringIterableForeachRunner"),
}

COMMANDS: list[dict[str, object]] = []


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write_status(prefix: str, status: int | str, stdout: bytes = b"", stderr: bytes = b"") -> None:
    (OUT / f"{prefix}.stdout").write_bytes(stdout)
    (OUT / f"{prefix}.stderr").write_bytes(stderr)
    (OUT / f"{prefix}.status").write_text(f"{status}\n", encoding="utf-8")


def run(prefix: str, args: list[str], cwd: Path | None = None) -> int:
    result = subprocess.run(args, cwd=cwd, capture_output=True, timeout=120)
    write_status(prefix, result.returncode, result.stdout, result.stderr)
    COMMANDS.append({"prefix": prefix, "argv": args, "cwd": str(cwd) if cwd else None,
                     "status": result.returncode})
    return result.returncode


def skipped(prefix: str, reason: str) -> None:
    write_status(prefix, "skipped", b"", (f"not run: {reason}\n").encode())
    COMMANDS.append({"prefix": prefix, "status": "skipped", "reason": reason})


def runtime_text(prefix: str) -> None:
    shutil.copyfile(OUT / f"{prefix}.stdout", OUT / f"{prefix}.txt")


def source_package(source: str) -> str:
    match = re.search(r"^package\s+([^;]+);", source, re.MULTILINE)
    return match.group(1) if match else ""


def compile_and_run(case_dir: Path, variant: str, source_path: Path | None,
                    runner_path: Path, class_name: str, runner_name: str,
                    package: str = "") -> dict[str, object]:
    compiled = case_dir / "compiled" / variant
    compiled.mkdir(parents=True, exist_ok=True)
    result: dict[str, object] = {}
    if source_path is None or not source_path.exists():
        skipped(f"{variant}-javac", f"{variant} class source was not produced")
        skipped(f"{variant}-runner-javac", f"{variant} class javac was skipped")
        skipped(f"{variant}-runtime", f"{variant} class or runner javac was skipped")
        return {"javac": "skipped", "runner_javac": "skipped", "runtime": "skipped"}

    status = run(f"{variant}-javac", ["javac", "--release", "8", "-g:none", "-d",
                                      str(compiled), str(source_path)])
    result["javac"] = status
    if status != 0:
        skipped(f"{variant}-runner-javac", f"{variant} class javac failed")
        skipped(f"{variant}-runtime", f"{variant} class javac failed")
        return {**result, "runner_javac": "skipped", "runtime": "skipped"}

    class_path = compiled / Path(*package.split(".")) / f"{class_name}.class"
    if not class_path.exists():
        skipped(f"{variant}-runner-javac", f"{variant} class file was not emitted")
        skipped(f"{variant}-runtime", f"{variant} class file was not emitted")
        return {**result, "runner_javac": "skipped", "runtime": "skipped"}
    result["class_bytes"] = class_path.stat().st_size
    result["class_sha256"] = sha256(class_path)
    if variant == "original":
        shutil.copyfile(class_path, case_dir / f"{class_name}.class")
        (case_dir / "original-class-sha256.txt").write_text(
            f"{result['class_sha256']}  {class_name}.class\n", encoding="utf-8")

    effective_runner = runner_path
    if package:
        effective_runner = case_dir / "compiled" / variant / f"{runner_name}.java"
        runner_text = runner_path.read_text(encoding="utf-8")
        effective_runner.write_text(f"package {package};\n\n{runner_text}", encoding="utf-8")
    runner_status = run(f"{variant}-runner-javac", ["javac", "--release", "8", "-g:none",
                                                      "-cp", str(compiled), "-d", str(compiled),
                                                      str(effective_runner)])
    result["runner_javac"] = runner_status
    if runner_status == 0:
        runtime_status = run(f"{variant}-runtime", ["java", "-Xverify:all", "-cp",
                                                      str(compiled),
                                                      f"{package + '.' if package else ''}{runner_name}"])
        runtime_text(f"{variant}-runtime")
        result["runtime"] = runtime_status
    else:
        skipped(f"{variant}-runtime", f"{variant} runner javac failed")
        result["runtime"] = "skipped"
    return result


def line_comparison(case_dir: Path, left: str, right: str) -> dict[str, object]:
    left_path = case_dir / f"{left}-runtime.txt"
    right_path = case_dir / f"{right}-runtime.txt"
    out = case_dir / f"comparison-{left}-vs-{right}.txt"
    if not left_path.exists() or not right_path.exists():
        out.write_text(f"comparison unavailable: {left} or {right} runtime did not run\n",
                       encoding="utf-8")
        return {"available": False}
    left_lines = left_path.read_text(encoding="utf-8").splitlines()
    right_lines = right_path.read_text(encoding="utf-8").splitlines()
    equal = left_lines == right_lines
    rows = [f"{left}: {len(left_lines)} lines; {right}: {len(right_lines)} lines; equal={str(equal).lower()}"]
    for index in range(max(len(left_lines), len(right_lines))):
        a = left_lines[index] if index < len(left_lines) else "<missing>"
        b = right_lines[index] if index < len(right_lines) else "<missing>"
        marker = "=" if a == b else "!"
        rows.append(f"{marker} {index + 1:03d} {left}={json.dumps(a, ensure_ascii=False)} {right}={json.dumps(b, ensure_ascii=False)}")
    out.write_text("\n".join(rows) + "\n", encoding="utf-8")
    return {"available": True, "equal": equal, "left_lines": len(left_lines),
            "right_lines": len(right_lines)}


def process_comparison(case_dir: Path, left: str, right: str, stage: str) -> dict[str, object]:
    left_prefix = f"{left}-{stage}"
    right_prefix = f"{right}-{stage}"
    out = case_dir / f"comparison-{stage}-{left}-vs-{right}.txt"
    required = [case_dir / f"{prefix}.{suffix}"
                for prefix in (left_prefix, right_prefix)
                for suffix in ("status", "stdout", "stderr")]
    if not all(path.exists() for path in required):
        out.write_text(f"comparison unavailable: {left} or {right} {stage} was not recorded\n",
                       encoding="utf-8")
        return {"available": False}
    rows = []
    equal = True
    for suffix in ("status", "stdout", "stderr"):
        a = (case_dir / f"{left_prefix}.{suffix}").read_text(encoding="utf-8").splitlines()
        b = (case_dir / f"{right_prefix}.{suffix}").read_text(encoding="utf-8").splitlines()
        same = a == b
        equal = equal and same
        rows.append(f"{suffix}: {left} {len(a)} lines; {right} {len(b)} lines; equal={str(same).lower()}")
        for index in range(max(len(a), len(b))):
            left_line = a[index] if index < len(a) else "<missing>"
            right_line = b[index] if index < len(b) else "<missing>"
            marker = "=" if left_line == right_line else "!"
            rows.append(f"{marker} {index + 1:03d} {left}={json.dumps(left_line, ensure_ascii=False)} {right}={json.dumps(right_line, ensure_ascii=False)}")
    out.write_text("\n".join(rows) + "\n", encoding="utf-8")
    return {"available": True, "equal": equal}


def audit_case(directory: str, class_name: str, runner_name: str) -> dict[str, object]:
    case_dir = OUT / directory
    case_dir.mkdir(parents=True, exist_ok=True)
    for name in ("original-runtime.txt", "jarde-runtime.txt", "jadx-runtime.txt"):
        path = OUT / name
        if path.exists():
            path.unlink()
    source_path = OUT / f"{class_name}.java"
    runner_path = OUT / f"{runner_name}.java"
    results: dict[str, object] = {}

    original = compile_and_run(case_dir, "original", source_path, runner_path,
                               class_name, runner_name)
    results["original"] = original
    original_class = case_dir / "compiled" / "original" / f"{class_name}.class"
    if original.get("javac") == 0:
        javap_status = run("original-javap", ["javap", "-v", "-c", "-p", str(original_class)])
        shutil.copyfile(OUT / "original-javap.stdout", case_dir / "original-javap.txt")
        shutil.copyfile(OUT / "original-javap.stderr", case_dir / "original-javap.stderr")
        (case_dir / "original-javap.status").write_text(f"{javap_status}\n", encoding="utf-8")
        results["original_javap"] = javap_status
    else:
        skipped("original-javap", "original class javac failed")
        results["original_javap"] = "skipped"

    generated = subprocess.run(
        [str(CLI), "class-source", "--input", str(original_class), "--class", class_name,
         "--policy", "single-class", "--release", "8", "--format", "text"],
        capture_output=True, timeout=120)
    write_status("jarde-cli", generated.returncode, generated.stdout, generated.stderr)
    (case_dir / "jarde.java.txt").write_bytes(generated.stdout)
    (case_dir / "jarde-report.txt").write_bytes(generated.stderr)
    (case_dir / "jarde-cli.status").write_text(f"{generated.returncode}\n", encoding="utf-8")
    COMMANDS.append({"prefix": "jarde-cli", "argv": [str(CLI), "class-source", "--input",
                     str(original_class), "--class", class_name, "--policy", "single-class",
                     "--release", "8", "--format", "text"], "status": generated.returncode})
    jarde_path = case_dir / "jarde-source.java"
    jarde_path.write_bytes(generated.stdout)
    jarde_source = generated.stdout.decode("utf-8", errors="replace")
    results["jarde_markers"] = generated.stdout.count(b"@bytecode")
    results["jarde"] = compile_and_run(case_dir, "jarde", jarde_path, runner_path,
                                       class_name, runner_name, source_package(jarde_source))

    jadx_dir: Path | None = None
    if original.get("javac") == 0:
        with tempfile.TemporaryDirectory(prefix=f"jarde-enhanced-{directory}-") as tmp:
            tmp_path = Path(tmp)
            jadx_status = run("jadx", ["jadx", "--no-res", "-d", str(tmp_path / "out"),
                                        str(original_class)])
            results["jadx"] = jadx_status
            jadx_sources = list((tmp_path / "out").rglob(f"{class_name}.java")) if (tmp_path / "out").exists() else []
            if jadx_status == 0 and jadx_sources:
                jadx_path = case_dir / "jadx.java.txt"
                shutil.copyfile(jadx_sources[0], jadx_path)
                jadx_source = jadx_path.read_text(encoding="utf-8")
                jadx_compile_path = case_dir / "compiled" / "jadx-source" / f"{class_name}.java"
                jadx_compile_path.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(jadx_path, jadx_compile_path)
                results["jadx_source"] = compile_and_run(case_dir, "jadx", jadx_compile_path,
                                                         runner_path, class_name, runner_name,
                                                         source_package(jadx_source))
            else:
                if jadx_status == 0:
                    skipped("jadx-javac", "JADX emitted no source for target class")
                    skipped("jadx-runner-javac", "JADX class javac was skipped")
                    skipped("jadx-runtime", "JADX class javac was skipped")
                else:
                    skipped("jadx-javac", "JADX command failed")
                    skipped("jadx-runner-javac", "JADX class javac was skipped")
                    skipped("jadx-runtime", "JADX class javac was skipped")
                results["jadx_source"] = {"javac": "skipped", "runner_javac": "skipped", "runtime": "skipped"}
            for suffix in ("stdout", "stderr", "status"):
                shutil.copyfile(OUT / f"jadx.{suffix}", case_dir / f"jadx.{suffix}")
    else:
        skipped("jadx", "original class javac failed")
        skipped("jadx-javac", "original class javac failed")
        skipped("jadx-runner-javac", "original class javac failed")
        skipped("jadx-runtime", "original class javac failed")
        results["jadx"] = "skipped"
        results["jadx_source"] = {"javac": "skipped", "runner_javac": "skipped", "runtime": "skipped"}

    for variant in ("original", "jarde", "jadx"):
        runtime = OUT / f"{variant}-runtime.txt"
        variant_result = results.get(variant)
        if not isinstance(variant_result, dict):
            variant_result = results.get(f"{variant}_source", {})
        if variant_result.get("runtime") == 0 and runtime.exists():
            shutil.copyfile(runtime, case_dir / runtime.name)
        for suffix in ("stdout", "stderr", "status"):
            path = OUT / f"{variant}-runtime.{suffix}"
            if path.exists():
                shutil.copyfile(path, case_dir / path.name)
    for prefix in ("original-javac", "original-runner-javac", "original-runtime",
                   "jarde-javac", "jarde-runner-javac", "jarde-runtime",
                   "jadx-javac", "jadx-runner-javac", "jadx-runtime"):
        for suffix in ("stdout", "stderr", "status"):
            path = OUT / f"{prefix}.{suffix}"
            if path.exists():
                shutil.copyfile(path, case_dir / path.name)

    results["line_comparison_jarde"] = line_comparison(case_dir, "original", "jarde")
    results["line_comparison_jadx"] = line_comparison(case_dir, "original", "jadx")
    for stage in ("javac", "runner-javac", "runtime"):
        results[f"process_comparison_{stage}_jarde"] = process_comparison(
            case_dir, "original", "jarde", stage)
        results[f"process_comparison_{stage}_jadx"] = process_comparison(
            case_dir, "original", "jadx", stage)
    return results


def main() -> None:
    for directory in CASES:
        case_dir = OUT / directory
        if case_dir.exists():
            shutil.rmtree(case_dir)
    for prefix in ("original-javac", "original-runner-javac", "original-runtime",
                   "jarde-javac", "jarde-runner-javac", "jarde-runtime",
                   "jadx-javac", "jadx-runner-javac", "jadx-runtime",
                   "original-javap", "jarde-cli", "jadx"):
        for suffix in ("stdout", "stderr", "status", "txt"):
            path = OUT / f"{prefix}.{suffix}"
            if path.exists():
                path.unlink()
    for name in ("audit.json", "cli-sha256-before.txt", "cli-sha256-after.txt"):
        path = OUT / name
        if path.exists():
            path.unlink()
    before = sha256(CLI)
    (OUT / "cli-sha256-before.txt").write_text(f"{before}  {CLI.name}\n", encoding="utf-8")
    if before != EXPECTED_CLI_SHA256:
        (OUT / "audit.json").write_text(json.dumps({"error": "frozen CLI hash mismatch",
            "actual": before, "expected": EXPECTED_CLI_SHA256}, indent=2) + "\n", encoding="utf-8")
        raise SystemExit(f"frozen CLI hash mismatch: {before}")

    results = {directory: audit_case(directory, class_name, runner_name)
               for directory, (class_name, runner_name) in CASES.items()}
    after = sha256(CLI)
    (OUT / "cli-sha256-after.txt").write_text(f"{after}  {CLI.name}\n", encoding="utf-8")
    summary = {"frozen_cli": str(CLI), "expected_cli_sha256": EXPECTED_CLI_SHA256,
               "before_sha256": before, "after_sha256": after,
               "cli_unchanged": before == after, "cases": results, "commands": COMMANDS}
    (OUT / "audit.json").write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n",
                                    encoding="utf-8")
    for prefix in ("original-javac", "original-runner-javac", "original-runtime",
                   "jarde-javac", "jarde-runner-javac", "jarde-runtime",
                   "jadx-javac", "jadx-runner-javac", "jadx-runtime",
                   "original-javap", "jarde-cli", "jadx"):
        for suffix in ("stdout", "stderr", "status", "txt"):
            path = OUT / f"{prefix}.{suffix}"
            if path.exists():
                path.unlink()
    if before != after:
        raise SystemExit("frozen CLI changed during audit")


if __name__ == "__main__":
    main()
