from __future__ import annotations

import hashlib
import json
import re
import shutil
import struct
import subprocess
import tempfile
from pathlib import Path


OUT = Path(__file__).resolve().parent
CLI = Path("/tmp/jarde-cli-deferred-accepted-7747")
EXPECTED_CLI_SHA256 = "7747b60a17dc635f0e8402d867cb9e74f9472a62057eb4865dbd8f4a207dfb34"
EXPECTED_FULL_CLASS_SHA256 = "5298588750f47fdd68525c1d570b2226f5d1c00230101e572b390c782073baf2"


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write_status(prefix: str, status: int | str) -> None:
    (OUT / f"{prefix}.status").write_text(f"{status}\n", encoding="utf-8")


def command(args: list[str], prefix: str, cwd: Path | None = None) -> int:
    result = subprocess.run(args, cwd=cwd, capture_output=True, text=True, timeout=90)
    (OUT / f"{prefix}.stdout").write_text(result.stdout, encoding="utf-8")
    (OUT / f"{prefix}.stderr").write_text(result.stderr, encoding="utf-8")
    write_status(prefix, result.returncode)
    return result.returncode


def alias_output(prefix: str, stdout_name: str) -> None:
    shutil.copy2(OUT / f"{prefix}.stdout", OUT / stdout_name)
    stderr_target = OUT / f"{stdout_name[:-4]}.stderr"
    status_target = OUT / f"{stdout_name[:-4]}.status"
    if stderr_target != OUT / f"{prefix}.stderr":
        shutil.copy2(OUT / f"{prefix}.stderr", stderr_target)
    if status_target != OUT / f"{prefix}.status":
        shutil.copy2(OUT / f"{prefix}.status", status_target)


def package_name(source: str) -> str:
    for line in source.splitlines():
        if line.startswith("package "):
            return line[len("package "):].rstrip(";")
    return ""


def with_package(source: str, package: str) -> str:
    if not package:
        return source
    return f"package {package};\n\n{source}"


def save_class_hash(prefix: str, class_file: Path) -> str:
    value = sha256(class_file)
    (OUT / f"{prefix}-class-sha256.txt").write_text(
        f"{value}  {class_file.name}\n", encoding="utf-8"
    )
    return value


def export_full_aliases() -> None:
    """Keep the short names used by the first audit run reproducible."""
    pairs = {
        "full-original-javac.stdout": "original-javac.stdout",
        "full-original-javac.stderr": "original-javac.stderr",
        "full-original-javac.status": "original-javac.status",
        "full-original-runner-javac.stdout": "original-runner-javac.stdout",
        "full-original-runner-javac.stderr": "original-runner-javac.stderr",
        "full-original-runner-javac.status": "original-runner-javac.status",
        "full-original-runtime.txt": "original-runtime.txt",
        "full-original-runtime.stderr": "original-runtime.stderr",
        "full-original-runtime.status": "original-runtime.status",
        "full-original-javap.txt": "original-javap.txt",
        "full-original-javap.stderr": "original-javap.stderr",
        "full-original-javap.status": "original-javap.status",
        "full-jarde.java.txt": "jarde.java.txt",
        "full-jarde-report.txt": "jarde-report.txt",
        "full-jarde-cli.status": "jarde-cli.status",
        "full-jarde-javac.stdout": "jarde-javac.stdout",
        "full-jarde-javac.stderr": "jarde-javac.stderr",
        "full-jarde-javac.status": "jarde-javac.status",
        "full-jarde-runtime.txt": "jarde-runtime.txt",
        "full-jarde-runtime.stderr": "jarde-runtime.stderr",
        "full-jarde-runtime.status": "jarde-runtime.status",
        "full-jadx.java.txt": "jadx.java.txt",
        "full-jadx.stdout": "jadx.stdout",
        "full-jadx.stderr": "jadx.stderr",
        "full-jadx.status": "jadx.status",
        "full-jadx-javac.stdout": "jadx-javac.stdout",
        "full-jadx-javac.stderr": "jadx-javac.stderr",
        "full-jadx-javac.status": "jadx-javac.status",
        "full-jadx-runtime.txt": "jadx-runtime.txt",
        "full-jadx-runtime.stderr": "jadx-runtime.stderr",
        "full-jadx-runtime.status": "jadx-runtime.status",
    }
    for source, destination in pairs.items():
        shutil.copy2(OUT / source, OUT / destination)


def run_variant(
    work: Path,
    prefix: str,
    class_name: str,
    runner_name: str,
    source_class: Path,
    source_runner: Path,
    freeze_class: bool,
) -> dict[str, object]:
    original = work / prefix / "original"
    original.mkdir(parents=True)
    class_status = command(
        [
            "javac",
            "--release",
            "8",
            "-g:none",
            "-d",
            str(original),
            str(source_class),
            str(source_runner),
        ],
        f"{prefix}-original-javac",
    )
    result: dict[str, object] = {"source_javac": class_status}
    class_file = original / f"{class_name}.class"
    if class_status != 0:
        return result
    class_hash = save_class_hash(prefix, class_file)
    result.update(
        {
            "class_bytes": class_file.stat().st_size,
            "class_sha256": class_hash,
        }
    )
    if freeze_class:
        frozen = OUT / f"{class_name}.class"
        if sha256(frozen) != class_hash:
            raise AssertionError("source compilation differs from frozen original class")
        if class_hash != EXPECTED_FULL_CLASS_SHA256:
            raise AssertionError("unexpected frozen original class hash")
    result["original_runner_javac"] = command(
        ["javac", "--release", "8", "-g:none", "-cp", str(original), "-d", str(original), str(source_runner)],
        f"{prefix}-original-runner-javac",
    )
    result["original_runtime"] = command(
        ["java", "-Xverify:all", "-cp", str(original), runner_name],
        f"{prefix}-original-runtime",
    )
    alias_output(f"{prefix}-original-runtime", f"{prefix}-original-runtime.txt")
    result["code_count"] = 0
    if command(["javap", "-v", "-c", "-p", str(class_file)], f"{prefix}-original-javap") == 0:
        alias_output(f"{prefix}-original-javap", f"{prefix}-original-javap.txt")
        javap = (OUT / f"{prefix}-original-javap.stdout").read_text(encoding="utf-8")
        result["code_count"] = len(re.findall(r"^    Code:$", javap, re.MULTILINE))

    cli_before = sha256(CLI)
    (OUT / "cli-sha256-before.txt").write_text(f"{cli_before}  jarde-cli\n", encoding="utf-8")
    if cli_before != EXPECTED_CLI_SHA256:
        raise AssertionError("frozen CLI hash is not the requested deferred-final-ecab build")
    recovered = work / prefix / "jarde"
    recovered.mkdir(parents=True)
    generated = subprocess.run(
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
    (OUT / f"{prefix}-jarde.java.txt").write_text(generated.stdout, encoding="utf-8")
    (OUT / f"{prefix}-jarde-report.txt").write_text(generated.stderr, encoding="utf-8")
    write_status(f"{prefix}-jarde-cli", generated.returncode)
    (recovered / f"{class_name}.java").write_text(generated.stdout, encoding="utf-8")
    result.update(
        {
            "jarde_cli": generated.returncode,
            "jarde_quotes": generated.stdout.count("@bytecode"),
        }
    )
    jarde_runner = recovered / f"{runner_name}.java"
    jarde_runner.write_text(with_package(source_runner.read_text(encoding="utf-8"), package_name(generated.stdout)), encoding="utf-8")
    jarde_compile = command(
        [
            "javac",
            "--release",
            "8",
            "-g:none",
            "-d",
            str(recovered / "classes"),
            str(recovered / f"{class_name}.java"),
            str(jarde_runner),
        ],
        f"{prefix}-jarde-javac",
    )
    result["jarde_javac"] = jarde_compile
    if jarde_compile == 0:
        result["jarde_runtime"] = command(
            ["java", "-Xverify:all", "-cp", str(recovered / "classes"), runner_name],
            f"{prefix}-jarde-runtime",
        )
        alias_output(f"{prefix}-jarde-runtime", f"{prefix}-jarde-runtime.txt")
        result["jarde_equal"] = (OUT / f"{prefix}-original-runtime.txt").read_bytes() == (OUT / f"{prefix}-jarde-runtime.txt").read_bytes()
    else:
        (OUT / f"{prefix}-jarde-runtime.txt").write_text("", encoding="utf-8")
        (OUT / f"{prefix}-jarde-runtime.stderr").write_text("not run: javac failed\n", encoding="utf-8")
        write_status(f"{prefix}-jarde-runtime", "skipped")

    jadx_dir = work / prefix / "jadx"
    result["jadx"] = command(["jadx", "--no-res", "-d", str(jadx_dir), str(class_file)], f"{prefix}-jadx")
    generated_jadx = next(jadx_dir.rglob(f"{class_name}.java"))
    jadx_source = generated_jadx.read_text(encoding="utf-8")
    (OUT / f"{prefix}-jadx.java.txt").write_text(jadx_source, encoding="utf-8")
    jadx_runner = work / prefix / "jadx-runner" / f"{runner_name}.java"
    jadx_runner.parent.mkdir(parents=True)
    jadx_runner.write_text(with_package(source_runner.read_text(encoding="utf-8"), package_name(jadx_source)), encoding="utf-8")
    jadx_classes = work / prefix / "jadx-classes"
    jadx_compile = command(
        ["javac", "--release", "8", "-g:none", "-d", str(jadx_classes), str(generated_jadx), str(jadx_runner)],
        f"{prefix}-jadx-javac",
    )
    result["jadx_javac"] = jadx_compile
    if jadx_compile == 0:
        package_prefix = package_name(jadx_source) + "." if package_name(jadx_source) else ""
        result["jadx_runtime"] = command(
            ["java", "-Xverify:all", "-cp", str(jadx_classes), package_prefix + runner_name],
            f"{prefix}-jadx-runtime",
        )
        alias_output(f"{prefix}-jadx-runtime", f"{prefix}-jadx-runtime.txt")
        result["jadx_equal"] = (OUT / f"{prefix}-original-runtime.txt").read_bytes() == (OUT / f"{prefix}-jadx-runtime.txt").read_bytes()

    cli_after = sha256(CLI)
    (OUT / "cli-sha256-after.txt").write_text(f"{cli_after}  jarde-cli\n", encoding="utf-8")
    result["cli_sha256"] = cli_after
    result["cli_unchanged"] = cli_before == cli_after
    return result


def run_patched_target(work: Path) -> dict[str, object]:
    original = work / "target" / "original"
    class_file = original / "ArrayReferenceOverloadTarget.class"
    javap = (OUT / "target-original-javap.txt").read_text(encoding="utf-8")
    checkcast_match = re.search(r'checkcast\s+#(\d+)\s+// class "\[Ljava/lang/Object;"', javap)
    method_match = re.search(
        r"invokestatic\s+#(\d+)\s+// Method overload:\(\[Ljava/lang/Object;\)Ljava/lang/String;",
        javap,
    )
    if checkcast_match is None or method_match is None:
        raise AssertionError("target javap did not expose the expected Object[] invocation")
    checkcast_cp = int(checkcast_match.group(1))
    method_cp = int(method_match.group(1))
    old_code = b"\x2a\xc0" + struct.pack(">H", checkcast_cp) + b"\xb8" + struct.pack(">H", method_cp) + b"\xb0"
    patched_code = b"\x2a\x00\x00\x00\xb8" + struct.pack(">H", method_cp) + b"\xb0"
    original_bytes = class_file.read_bytes()
    if original_bytes.count(old_code) != 1:
        raise AssertionError("target checkcast sequence was not uniquely identified")
    patched_bytes = original_bytes.replace(old_code, patched_code)
    patched_dir = work / "target-patched"
    patched_dir.mkdir(parents=True)
    for class_path in original.glob("*.class"):
        shutil.copy2(class_path, patched_dir / class_path.name)
    patched_class = patched_dir / class_file.name
    patched_class.write_bytes(patched_bytes)
    patch_record = {
        "checkcast_constant_pool_index": checkcast_cp,
        "object_array_method_constant_pool_index": method_cp,
        "method_info_code_before": old_code.hex(),
        "method_info_code_after": patched_code.hex(),
        "patched_offset": original_bytes.index(old_code) + 1,
        "original_class_sha256": sha256(class_file),
        "patched_class_sha256": sha256(patched_class),
    }
    (OUT / "target-patch.json").write_text(json.dumps(patch_record, indent=2) + "\n", encoding="utf-8")
    result: dict[str, object] = {"patch": patch_record}
    result["patched_javap"] = command(["javap", "-v", "-c", "-p", str(patched_class)], "target-patched-javap")
    alias_output("target-patched-javap", "target-patched-javap.txt")
    result["patched_runtime"] = command(
        ["java", "-Xverify:all", "-cp", str(patched_dir), "ArrayReferenceOverloadTargetRunner"],
        "target-patched-runtime",
    )
    alias_output("target-patched-runtime", "target-patched-runtime.txt")
    result["patched_runtime_equal"] = (OUT / "target-patched-runtime.txt").read_bytes() == (OUT / "target-original-runtime.txt").read_bytes()

    generated = subprocess.run(
        [
            str(CLI),
            "class-source",
            "--input",
            str(patched_class),
            "--class",
            "ArrayReferenceOverloadTarget",
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
    (OUT / "target-patched-jarde.java.txt").write_text(generated.stdout, encoding="utf-8")
    (OUT / "target-patched-jarde-report.txt").write_text(generated.stderr, encoding="utf-8")
    write_status("target-patched-jarde-cli", generated.returncode)
    recovered = work / "target-patched-jarde"
    recovered.mkdir(parents=True)
    (recovered / "ArrayReferenceOverloadTarget.java").write_text(generated.stdout, encoding="utf-8")
    runner = recovered / "ArrayReferenceOverloadTargetRunner.java"
    runner.write_text((OUT / "ArrayReferenceOverloadTargetRunner.java").read_text(encoding="utf-8"), encoding="utf-8")
    result["jarde_cli"] = generated.returncode
    result["jarde_quotes"] = generated.stdout.count("@bytecode")
    result["jarde_javac"] = command(
        [
            "javac",
            "--release",
            "8",
            "-g:none",
            "-d",
            str(recovered / "classes"),
            str(recovered / "ArrayReferenceOverloadTarget.java"),
            str(runner),
        ],
        "target-patched-jarde-javac",
    )
    if result["jarde_javac"] == 0:
        result["jarde_runtime"] = command(
            ["java", "-Xverify:all", "-cp", str(recovered / "classes"), "ArrayReferenceOverloadTargetRunner"],
            "target-patched-jarde-runtime",
        )
        alias_output("target-patched-jarde-runtime", "target-patched-jarde-runtime.txt")
        result["jarde_equal_patched"] = (OUT / "target-patched-jarde-runtime.txt").read_bytes() == (OUT / "target-patched-runtime.txt").read_bytes()
    else:
        (OUT / "target-patched-jarde-runtime.txt").write_text("", encoding="utf-8")
        (OUT / "target-patched-jarde-runtime.stderr").write_text("not run: javac failed\n", encoding="utf-8")
        write_status("target-patched-jarde-runtime", "skipped")
    return result


def main() -> None:
    cli_start = sha256(CLI)
    (OUT / "cli-sha256-start.txt").write_text(f"{cli_start}  jarde-cli\n", encoding="utf-8")
    if cli_start != EXPECTED_CLI_SHA256:
        raise AssertionError("requested CLI is unavailable or has changed")
    with tempfile.TemporaryDirectory(prefix="jarde-array-reference-conversion-") as temporary:
        work = Path(temporary)
        full = run_variant(
            work,
            "full",
            "ArrayReferenceConversions",
            "ArrayReferenceConversionsRunner",
            OUT / "ArrayReferenceConversions.java",
            OUT / "ArrayReferenceConversionsRunner.java",
            True,
        )
        export_full_aliases()
        core = run_variant(
            work,
            "core",
            "ArrayReferenceCore",
            "ArrayReferenceCoreRunner",
            OUT / "ArrayReferenceCore.java",
            OUT / "ArrayReferenceCoreRunner.java",
            False,
        )
        target = run_variant(
            work,
            "target",
            "ArrayReferenceOverloadTarget",
            "ArrayReferenceOverloadTargetRunner",
            OUT / "ArrayReferenceOverloadTarget.java",
            OUT / "ArrayReferenceOverloadTargetRunner.java",
            False,
        )
        target["patched_variant"] = run_patched_target(work)
    summary = {
        "cli_sha256": cli_start,
        "full": full,
        "core": core,
        "target_selection": target,
        "full_runtime_lines": len((OUT / "full-original-runtime.txt").read_text(encoding="utf-8").splitlines()),
        "core_runtime_lines": len((OUT / "core-original-runtime.txt").read_text(encoding="utf-8").splitlines()),
        "target_runtime_lines": len((OUT / "target-original-runtime.txt").read_text(encoding="utf-8").splitlines()),
    }
    (OUT / "summary.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
