from __future__ import annotations

import hashlib
import json
import re
import shutil
import subprocess
import tempfile
from pathlib import Path


OUT = Path(__file__).resolve().parent
CLI = Path("/tmp/jarde-cli-deferred-accepted-7747")
EXPECTED_CLI_SHA256 = "7747b60a17dc635f0e8402d867cb9e74f9472a62057eb4865dbd8f4a207dfb34"
CLASS_NAME = "ArrayInitializerProbe"
RUNNER_NAME = "ArrayInitializerRunner"


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def record_command(prefix: str, args: list[str], cwd: Path | None = None) -> int:
    result = subprocess.run(args, cwd=cwd, capture_output=True, text=True, timeout=90)
    (OUT / f"{prefix}.stdout").write_text(result.stdout, encoding="utf-8")
    (OUT / f"{prefix}.stderr").write_text(result.stderr, encoding="utf-8")
    (OUT / f"{prefix}.status").write_text(f"{result.returncode}\n", encoding="utf-8")
    return result.returncode


def copy_runtime_alias(prefix: str) -> None:
    shutil.copy2(OUT / f"{prefix}.stdout", OUT / f"{prefix}.txt")


def package_name(source: str) -> str:
    match = re.search(r"^package\s+([^;]+);", source, re.MULTILINE)
    return match.group(1) if match else ""


def runner_source(package: str) -> str:
    source = (OUT / f"{RUNNER_NAME}.java").read_text(encoding="utf-8")
    return f"package {package};\n\n{source}" if package else source


def compile_runner(
    prefix: str,
    runner_source_path: Path,
    classpath: Path,
    output: Path,
) -> int:
    return record_command(
        prefix,
        [
            "javac",
            "--release",
            "8",
            "-g:none",
            "-cp",
            str(classpath),
            "-d",
            str(output),
            str(runner_source_path),
        ],
    )


def run_variant(
    work: Path,
    variant: str,
    class_source: Path,
    class_package: str = "",
) -> dict[str, object]:
    classes = work / variant / "classes"
    classes.mkdir(parents=True)
    class_status = record_command(
        f"{variant}-javac",
        [
            "javac",
            "--release",
            "8",
            "-g:none",
            "-d",
            str(classes),
            str(class_source),
        ],
    )
    result: dict[str, object] = {"class_javac": class_status}
    class_file = classes / Path(*class_package.split(".")) / f"{CLASS_NAME}.class"
    if class_status != 0:
        for skipped in ("runner-javac", "runtime"):
            (OUT / f"{variant}-{skipped}.stdout").write_text("", encoding="utf-8")
            (OUT / f"{variant}-{skipped}.stderr").write_text(
                f"not run: {variant} class javac failed\n", encoding="utf-8"
            )
            (OUT / f"{variant}-{skipped}.status").write_text("skipped\n", encoding="utf-8")
        return result

    class_hash = sha256(class_file)
    result.update({"class_bytes": class_file.stat().st_size, "class_sha256": class_hash})
    (OUT / f"{variant}-class-sha256.txt").write_text(
        f"{class_hash}  {CLASS_NAME}.class\n", encoding="utf-8"
    )
    if variant == "original":
        shutil.copy2(class_file, OUT / f"{CLASS_NAME}.class")

    runner_path = work / variant / f"{RUNNER_NAME}.java"
    runner_path.write_text(runner_source(class_package), encoding="utf-8")
    runner_status = compile_runner(
        f"{variant}-runner-javac", runner_path, classes, classes
    )
    result["runner_javac"] = runner_status
    if runner_status == 0:
        runtime_status = record_command(
            f"{variant}-runtime",
            ["java", "-Xverify:all", "-cp", str(classes), f"{class_package + '.' if class_package else ''}{RUNNER_NAME}"],
        )
        copy_runtime_alias(f"{variant}-runtime")
        result["runtime"] = runtime_status
    else:
        (OUT / f"{variant}-runtime.txt").write_text("", encoding="utf-8")
        (OUT / f"{variant}-runtime.stderr").write_text("not run: runner javac failed\n", encoding="utf-8")
        (OUT / f"{variant}-runtime.status").write_text("skipped\n", encoding="utf-8")
        result["runtime"] = "skipped"
    return result


def main() -> None:
    source_class = OUT / f"{CLASS_NAME}.java"
    source_runner = OUT / f"{RUNNER_NAME}.java"
    cli_hash = sha256(CLI)
    (OUT / "cli-sha256-before.txt").write_text(f"{cli_hash}  {CLI.name}\n", encoding="utf-8")
    if cli_hash != EXPECTED_CLI_SHA256:
        raise SystemExit(f"frozen CLI hash mismatch: {cli_hash}")

    with tempfile.TemporaryDirectory(prefix="jarde-array-initializers-") as temporary:
        work = Path(temporary)
        results: dict[str, object] = {}
        results["original"] = run_variant(work, "original", source_class)

        original_class = work / "original" / "classes" / f"{CLASS_NAME}.class"
        if not original_class.exists():
            raise SystemExit("original class did not compile")
        javap_status = record_command(
            "original-javap",
            ["javap", "-v", "-c", "-p", str(original_class)],
        )
        shutil.copy2(OUT / "original-javap.stdout", OUT / "original-javap.txt")
        results["original_javap"] = javap_status
        javap = (OUT / "original-javap.stdout").read_text(encoding="utf-8")
        results["code_attributes"] = len(re.findall(r"^    Code:$", javap, re.MULTILINE))

        generated = subprocess.run(
            [
                str(CLI),
                "class-source",
                "--input",
                str(original_class),
                "--class",
                CLASS_NAME,
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
        (OUT / "jarde.java.txt").write_text(generated.stdout, encoding="utf-8")
        (OUT / "jarde-report.txt").write_text(generated.stderr, encoding="utf-8")
        (OUT / "jarde-cli.status").write_text(f"{generated.returncode}\n", encoding="utf-8")
        jarde_source = work / "jarde" / f"{CLASS_NAME}.java"
        jarde_source.parent.mkdir()
        jarde_source.write_text(generated.stdout, encoding="utf-8")
        results["jarde_cli"] = generated.returncode
        results["jarde_markers"] = generated.stdout.count("@bytecode")
        results["jarde"] = run_variant(
            work, "jarde", jarde_source, package_name(generated.stdout)
        )

        jadx_dir = work / "jadx"
        jadx_status = record_command(
            "jadx", ["jadx", "--no-res", "-d", str(jadx_dir), str(original_class)]
        )
        results["jadx"] = jadx_status
        if jadx_status == 0:
            jadx_source = next(jadx_dir.rglob(f"{CLASS_NAME}.java"))
            jadx_text = jadx_source.read_text(encoding="utf-8")
            (OUT / "jadx.java.txt").write_text(jadx_text, encoding="utf-8")
            results["jadx_source"] = run_variant(
                work, "jadx", jadx_source, package_name(jadx_text)
            )

        final_cli_hash = sha256(CLI)
        (OUT / "cli-sha256-after.txt").write_text(
            f"{final_cli_hash}  {CLI.name}\n", encoding="utf-8"
        )
        results["cli_sha256_after"] = final_cli_hash
        results["cli_unchanged"] = cli_hash == final_cli_hash

        original_runtime = (OUT / "original-runtime.txt").read_bytes()
        for name in ("jarde", "jadx"):
            runtime = OUT / f"{name}-runtime.txt"
            status = (OUT / f"{name}-runtime.status").read_text(encoding="utf-8").strip()
            results[f"{name}_runtime_equal"] = (
                "skipped" if status == "skipped" else runtime.exists() and runtime.read_bytes() == original_runtime
            )
        results["original_runtime_sha256"] = hashlib.sha256(original_runtime).hexdigest()
        results["source_lines"] = {
            "jarde": len(generated.stdout.splitlines()),
            "jadx": len((OUT / "jadx.java.txt").read_text(encoding="utf-8").splitlines()) if (OUT / "jadx.java.txt").exists() else None,
        }

    (OUT / "audit.json").write_text(json.dumps(results, indent=2, sort_keys=True) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
