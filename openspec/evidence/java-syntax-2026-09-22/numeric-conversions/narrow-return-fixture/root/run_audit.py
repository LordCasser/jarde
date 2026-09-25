from pathlib import Path
import hashlib
import json
import re
import shutil
import subprocess
import tempfile


ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
FIXTURE = Path(__file__).resolve().parent / "inputs"
EVIDENCE = Path(__file__).resolve().parent
PATCHER = EVIDENCE / "patch_descriptors.py"
TARGET_NAME = "NarrowIntegerReturns"
TARGET_SOURCE = FIXTURE / f"{TARGET_NAME}.java"
RUNNER_SOURCE = FIXTURE / "NarrowIntegerReturnsRunner.java"
FROZEN = FIXTURE / "v8/NarrowIntegerReturns.class"


def run(args, log, cwd=None):
    result = subprocess.run(args, cwd=cwd, capture_output=True, text=True, timeout=60)
    log.write_text(result.stdout + result.stderr)
    return result


def capture(args, cwd=None):
    return subprocess.run(args, cwd=cwd, capture_output=True, text=True, timeout=60)


def package_line(source):
    return next((line for line in source.splitlines() if line.startswith("package ")), "")


def compile_runner(directory, package_declaration=""):
    prefix = package_declaration + "\n" if package_declaration else ""
    runner = directory / "NarrowIntegerReturnsRunner.java"
    runner.write_text(prefix + RUNNER_SOURCE.read_text())
    return subprocess.run(
        [
            "javac",
            "--release",
            "8",
            "-g:none",
            "-cp",
            str(directory),
            "-d",
            str(directory),
            str(runner),
        ],
        cwd=directory,
        capture_output=True,
        text=True,
        timeout=60,
    )


def compile_generated(directory, target_source, package_declaration=""):
    prefix = package_declaration + "\n" if package_declaration else ""
    runner = directory / "NarrowIntegerReturnsRunner.java"
    runner.write_text(prefix + RUNNER_SOURCE.read_text())
    return subprocess.run(
        [
            "javac",
            "--release",
            "8",
            "-g:none",
            "-d",
            str(directory),
            str(target_source),
            str(runner),
        ],
        cwd=directory,
        capture_output=True,
        text=True,
        timeout=60,
    )


def main():
    EVIDENCE.mkdir(parents=True, exist_ok=True)
    cli_path = Path("/tmp/jarde-cli-deferred-interim-948a")
    cli_start = hashlib.sha256(cli_path.read_bytes()).hexdigest()
    (EVIDENCE / "cli-hash-input.txt").write_text(f"{cli_start}  target/debug/jarde-cli\n")
    (EVIDENCE / "source.java.txt").write_text(TARGET_SOURCE.read_text())

    with tempfile.TemporaryDirectory(prefix="jarde-narrow-return-fixture-") as temp:
        work = Path(temp)
        source = work / "source"
        source.mkdir()
        source_compile = run(
            [
                "javac",
                "--release",
                "8",
                "-g:none",
                "-d",
                str(source),
                str(TARGET_SOURCE),
            ],
            EVIDENCE / "source-javac.log",
        )
        (EVIDENCE / "source-javac-status.txt").write_text(f"exit={source_compile.returncode}\n")
        if source_compile.returncode != 0:
            raise SystemExit(source_compile.returncode)

        patched = work / "patched"
        patched.mkdir()
        patch_record = work / "descriptor-patches.json"
        patch_result = run(
            [
                "python3",
                str(PATCHER),
                str(source / f"{TARGET_NAME}.class"),
                str(patched / f"{TARGET_NAME}.class"),
                str(patch_record),
            ],
            EVIDENCE / "descriptor-patch.log",
        )
        (EVIDENCE / "descriptor-patch-status.txt").write_text(f"exit={patch_result.returncode}\n")
        if patch_result.returncode != 0:
            raise SystemExit(patch_result.returncode)
        shutil.copyfile(patch_record, EVIDENCE / "descriptor-patches.json")
        patch_metadata = json.loads(patch_record.read_text())

        assert (patched / f"{TARGET_NAME}.class").read_bytes() == FROZEN.read_bytes(), "fresh compiler/patcher output differs from frozen class"
        assert patch_metadata["code_sha256_before"] == patch_metadata["code_sha256_after"]

        original = work / "original"
        original.mkdir()
        shutil.copyfile(patched / f"{TARGET_NAME}.class", original / f"{TARGET_NAME}.class")
        runner_compile = compile_runner(original)
        (EVIDENCE / "runner-javac.log").write_text(runner_compile.stdout + runner_compile.stderr)
        (EVIDENCE / "runner-javac-status.txt").write_text(f"exit={runner_compile.returncode}\n")
        if runner_compile.returncode != 0:
            raise SystemExit(runner_compile.returncode)

        original_class = original / f"{TARGET_NAME}.class"
        assert original_class.read_bytes() == FROZEN.read_bytes(), "runner compilation overwrote frozen input"
        original_run = capture(
            ["java", "-Xverify:all", "-cp", str(original), "NarrowIntegerReturnsRunner"],
            cwd=original,
        )
        (EVIDENCE / "original.txt").write_text(original_run.stdout + original_run.stderr)
        (EVIDENCE / "original-status.txt").write_text(f"exit={original_run.returncode}\n")
        run(["javap", "-p", "-c", "-v", str(original_class)], EVIDENCE / "javap.txt")
        (EVIDENCE / "original-class-sha256.txt").write_text(
            f"{hashlib.sha256(original_class.read_bytes()).hexdigest()}  original/{TARGET_NAME}.class\n"
        )

        cli = capture(
            [
                str(cli_path),
                "class-source",
                "--input",
                str(original_class),
                "--class",
                TARGET_NAME,
                "--policy",
                "single-class",
                "--release",
                "8",
                "--format",
                "text",
                "--evidence",
                "all",
            ]
        )
        (EVIDENCE / "jarde.java.txt").write_text(cli.stdout)
        (EVIDENCE / "jarde-report.txt").write_text(f"exit={cli.returncode}\n" + cli.stderr)
        cli_end = hashlib.sha256(cli_path.read_bytes()).hexdigest()
        (EVIDENCE / "cli-hash-end.txt").write_text(f"{cli_end}  target/debug/jarde-cli\n")

        jarde = work / "jarde"
        jarde.mkdir()
        jarde_source = jarde / f"{TARGET_NAME}.java"
        jarde_source.write_text(cli.stdout)
        jarde_runner_compile = compile_generated(jarde, jarde_source)
        (EVIDENCE / "jarde-javac.log").write_text(jarde_runner_compile.stdout + jarde_runner_compile.stderr)
        (EVIDENCE / "jarde-javac-status.txt").write_text(f"exit={jarde_runner_compile.returncode}\n")
        jarde_run = None
        if jarde_runner_compile.returncode == 0:
            jarde_run = capture(
                ["java", "-Xverify:all", "-cp", str(jarde), "NarrowIntegerReturnsRunner"],
                cwd=jarde,
            )
            (EVIDENCE / "jarde.txt").write_text(jarde_run.stdout + jarde_run.stderr)
            (EVIDENCE / "jarde-status.txt").write_text(f"exit={jarde_run.returncode}\n")

        jadx = work / "jadx"
        jadx_tool = run(["jadx", "--no-res", "-d", str(jadx), str(original_class)], EVIDENCE / "jadx.log")
        jadx_source = next(jadx.rglob(f"{TARGET_NAME}.java"), None) if jadx_tool.returncode == 0 else None
        jadx_compile = None
        jadx_run = None
        jadx_package = ""
        if jadx_source is not None:
            generated = jadx_source.read_text()
            (EVIDENCE / "jadx.java.txt").write_text(generated)
            jadx_package = package_line(generated)
            jadx_input = work / "jadx-input"
            jadx_input.mkdir()
            jadx_source_copy = jadx_input / f"{TARGET_NAME}.java"
            jadx_source_copy.write_text(generated)
            jadx_runner_compile = compile_generated(jadx_input, jadx_source_copy, jadx_package)
            (EVIDENCE / "jadx-javac.log").write_text(
                jadx_runner_compile.stdout + jadx_runner_compile.stderr
            )
            (EVIDENCE / "jadx-javac-status.txt").write_text(
                f"exit={jadx_runner_compile.returncode}\n"
            )
            jadx_compile = jadx_runner_compile
            if jadx_compile.returncode == 0:
                package_name = jadx_package.removeprefix("package ").removesuffix(";")
                main_class = f"{package_name}.NarrowIntegerReturnsRunner" if package_name else "NarrowIntegerReturnsRunner"
                jadx_run = capture(
                    ["java", "-Xverify:all", "-cp", str(jadx_input), main_class],
                    cwd=jadx_input,
                )
                (EVIDENCE / "jadx.txt").write_text(jadx_run.stdout + jadx_run.stderr)
                (EVIDENCE / "jadx-status.txt").write_text(f"exit={jadx_run.returncode}\n")

        javap_text = (EVIDENCE / "javap.txt").read_text()
        conversion_opcodes = [
            "i2l", "i2f", "i2d", "l2i", "l2f", "l2d", "f2i", "f2l", "f2d",
            "d2i", "d2l", "d2f", "i2b", "i2c", "i2s",
        ]
        opcode_counts = {
            opcode: len(re.findall(rf"^\s*\d+:\s+{opcode}\b", javap_text, re.MULTILINE))
            for opcode in conversion_opcodes
        }
        original_output = (EVIDENCE / "original.txt").read_text()
        summary = {
            "class_sha256": hashlib.sha256(original_class.read_bytes()).hexdigest(),
            "class_bytes": len(original_class.read_bytes()),
            "methods": int(re.search(r"interfaces: 0, fields: 1, methods: (\d+)", javap_text).group(1)),
            "code_attributes": len(re.findall(r"^    Code:$", javap_text, re.MULTILINE)),
            "conversion_opcodes": opcode_counts,
            "code_attributes_before": patch_metadata["code_attributes"],
            "code_sha256_before": patch_metadata["code_sha256_before"],
            "code_sha256_after": patch_metadata["code_sha256_after"],
            "descriptor_patch_count": len(patch_metadata["patches"]),
            "runtime_cases": len(original_output.splitlines()),
            "original_exit": original_run.returncode,
            "jarde_cli_exit": cli.returncode,
            "jarde_quote_count": cli.stdout.count("@bytecode"),
            "jarde_javac_exit": jarde_runner_compile.returncode,
            "jarde_run_exit": jarde_run.returncode if jarde_run else None,
            "jadx_exit": jadx_tool.returncode,
            "jadx_javac_exit": jadx_compile.returncode if jadx_compile else None,
            "jadx_run_exit": jadx_run.returncode if jadx_run else None,
            "cli_hash_input": cli_start,
            "cli_hash_end": cli_end,
            "cli_hash_unchanged": cli_start == cli_end,
        }
        assert original_run.returncode == 0 and len(original_output.splitlines()) == 49
        assert all(n == 0 for n in opcode_counts.values())
        summary["frozen_class_equal"] = True
        summary["jarde_equals_original"] = jarde_run.stdout == original_run.stdout if jarde_run else None
        summary["jadx_equals_original"] = jadx_run.stdout == original_run.stdout if jadx_run else None
        (EVIDENCE / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
        assert cli_start == cli_end, "jarde CLI changed during the audit; rerun in a fresh work directory"


if __name__ == "__main__":
    main()
