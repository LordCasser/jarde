from __future__ import annotations

import hashlib
import json
import re
import subprocess
import tempfile
from pathlib import Path


OUT = Path(__file__).resolve().parent
CLI = Path("/tmp/jarde-cli-deferred-accepted-7747")
EXPECTED_CLI_SHA256 = "7747b60a17dc635f0e8402d867cb9e74f9472a62057eb4865dbd8f4a207dfb34"
CLASS = "ArrayInitializerBoundaries"
RUNNER = "ArrayInitializerBoundariesRunner"


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def command(name: str, args: list[str], cwd: Path | None = None) -> int:
    result = subprocess.run(args, cwd=cwd, capture_output=True, text=True, timeout=90)
    (OUT / f"{name}.stdout").write_text(result.stdout, encoding="utf-8")
    (OUT / f"{name}.stderr").write_text(result.stderr, encoding="utf-8")
    (OUT / f"{name}.status").write_text(f"{result.returncode}\n", encoding="utf-8")
    return result.returncode


def package(source: str) -> str:
    match = re.search(r"^package\s+([^;]+);", source, re.MULTILINE)
    return match.group(1) if match else ""


def class_compile(name: str, source: Path, output: Path) -> int:
    output.mkdir(parents=True, exist_ok=True)
    return command(name, ["javac", "--release", "8", "-g:none", "-d", str(output), str(source)])


def run_variant(name: str, source: Path, package_name: str, work: Path) -> dict[str, object]:
    classes = work / f"{name}-classes"
    status = class_compile(f"{name}-javac", source, classes)
    result: dict[str, object] = {"javac": status}
    class_file = classes / Path(*package_name.split(".")) / f"{CLASS}.class"
    if status:
        return result
    result["class_bytes"] = class_file.stat().st_size
    result["class_sha256"] = sha(class_file)
    if name == "original":
        command("original-javap", ["javap", "-v", "-c", "-p", str(class_file)])
        (OUT / "original-javap.txt").write_text((OUT / "original-javap.stdout").read_text(), encoding="utf-8")
    if name == "jarde":
        (OUT / "jarde-compiled-class-sha256.txt").write_text(f"{sha(class_file)}  {CLASS}.class\n", encoding="utf-8")
    runner_source = work / f"{RUNNER}.java"
    runner = (OUT / f"{RUNNER}.java").read_text(encoding="utf-8")
    runner_source.write_text((f"package {package_name};\n\n" if package_name else "") + runner, encoding="utf-8")
    runner_classes = work / f"{name}-runner-classes"
    runner_classes.mkdir()
    runner_status = command(f"{name}-runner-javac", ["javac", "--release", "8", "-g:none", "-cp", str(classes), "-d", str(runner_classes), str(runner_source)])
    result["runner_javac"] = runner_status
    if runner_status == 0:
        run_status = command(f"{name}-runtime", ["java", "-Xverify:all", "-cp", f"{runner_classes}:{classes}", f"{package_name + '.' if package_name else ''}{RUNNER}"])
        result["runtime"] = run_status
        if name == "original":
            (OUT / "original-runtime.txt").write_text((OUT / "original-runtime.stdout").read_text(), encoding="utf-8")
    else:
        result["runtime"] = "skipped"
    return result


def main() -> None:
    source_file = OUT / f"{CLASS}.java"
    runner_file = OUT / f"{RUNNER}.java"
    (OUT / "input-source-sha256.txt").write_text(
        f"{sha(source_file)}  {source_file.name}\n{sha(runner_file)}  {runner_file.name}\n",
        encoding="utf-8",
    )
    cli_hash = sha(CLI)
    (OUT / "cli-sha256-before.txt").write_text(f"{cli_hash}  {CLI.name}\n", encoding="utf-8")
    if cli_hash != EXPECTED_CLI_SHA256:
        raise SystemExit(f"frozen CLI hash mismatch: {cli_hash}")
    results: dict[str, object] = {}
    with tempfile.TemporaryDirectory(prefix="jarde-array-boundaries-") as tmp:
        work = Path(tmp)
        results["original"] = run_variant("original", source_file, "", work)
        original_class = work / "original-classes" / f"{CLASS}.class"
        cli = subprocess.run([str(CLI), "class-source", "--input", str(original_class), "--class", CLASS, "--policy", "single-class", "--release", "8", "--format", "text"], capture_output=True, text=True, timeout=90)
        (OUT / "jarde.java.txt").write_text(cli.stdout, encoding="utf-8")
        (OUT / "jarde-report.txt").write_text(cli.stderr, encoding="utf-8")
        (OUT / "jarde-cli.status").write_text(f"{cli.returncode}\n", encoding="utf-8")
        jarde_source = work / f"{CLASS}.java"
        jarde_source.write_text(cli.stdout, encoding="utf-8")
        results["jarde_cli"] = cli.returncode
        results["jarde_markers"] = cli.stdout.count("@bytecode")
        results["jarde"] = run_variant("jarde", jarde_source, package(cli.stdout), work)

        jadx_dir = work / "jadx"
        jadx_status = command("jadx", ["jadx", "--no-res", "-d", str(jadx_dir), str(original_class)])
        results["jadx"] = jadx_status
        if jadx_status == 0:
            jadx_source = next(jadx_dir.rglob(f"{CLASS}.java"))
            jadx_text = jadx_source.read_text(encoding="utf-8")
            (OUT / "jadx.java.txt").write_text(jadx_text, encoding="utf-8")
            results["jadx_source"] = run_variant("jadx", jadx_source, package(jadx_text), work)

        final = sha(CLI)
        (OUT / "cli-sha256-after.txt").write_text(f"{final}  {CLI.name}\n", encoding="utf-8")
        results["cli_unchanged"] = final == cli_hash
        original_file = OUT / "original-runtime.stdout"
        original_out = original_file.read_bytes() if original_file.exists() else b""
        results["jadx_runtime_equal"] = (OUT / "jadx-runtime.stdout").read_bytes() == original_out if results.get("jadx_source", {}).get("runtime") == 0 and original_file.exists() else "skipped"
        results["jarde_runtime_equal"] = (OUT / "jarde-runtime.stdout").read_bytes() == original_out if results.get("jarde", {}).get("runtime") == 0 and original_file.exists() else "skipped"
        results["original_code_attributes"] = len(re.findall(r"^    Code:$", (OUT / "original-javap.txt").read_text(encoding="utf-8"), re.MULTILINE))
        results["original_class_sha256"] = sha(original_class)
    (OUT / "summary.json").write_text(json.dumps(results, indent=2, sort_keys=True) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
